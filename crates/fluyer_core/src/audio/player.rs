use crate::events::EventSink;
use crate::metadata::MusicMetadata;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::path::PathBuf;
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use super::bass::*;

#[derive(Clone, Debug)]
pub struct TrackItem {
    pub metadata: MusicMetadata,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum RepeatMode {
    #[default]
    None = 0,
    All = 1,
    One = 2,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicPlayerSync {
    pub index: i64,
    pub current_position: Option<f64>,
    pub duration: Option<f64>,
    pub is_playing: bool,
    pub repeat_mode: RepeatMode,
    pub is_shuffled: bool,
}

impl MusicPlayerSync {
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn position_ms(&self) -> u64 {
        self.current_position
            .map(|s| (s * 1000.0) as u64)
            .unwrap_or(0)
    }

    pub fn duration_ms(&self) -> u64 {
        self.duration
            .map(|s| (s * 1000.0) as u64)
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, Default)]
struct PlayerState {
    track: Vec<TrackItem>,
    original_track: Option<Vec<TrackItem>>,
    current_index: Option<usize>,
    repeat_mode: RepeatMode,
}

impl PlayerState {
    fn get_next_index(&mut self, from_user: bool) -> Option<usize> {
        match (self.current_index, self.repeat_mode) {
            (Some(current), RepeatMode::One) if !from_user => Some(current),
            (Some(current), _) => {
                if current + 1 < self.track.len() {
                    Some(current + 1)
                } else if self.repeat_mode == RepeatMode::All {
                    Some(0)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub struct MusicPlayer {
    bass_mixer: Arc<AtomicU32>,
    current_stream: Arc<AtomicU32>,
    state: Arc<Mutex<PlayerState>>,
    temp_wav_path: Arc<Mutex<Option<PathBuf>>>,
    event_sink: Option<Arc<dyn EventSink>>,
}

struct SyncData {
    bass_mixer: Arc<AtomicU32>,
    current_stream: Arc<AtomicU32>,
    state: Arc<Mutex<PlayerState>>,
    temp_wav_path: Arc<Mutex<Option<PathBuf>>>,
    event_sink: Option<Arc<dyn EventSink>>,
}

extern "C" fn end_sync_callback(
    _handle: u32,
    _channel: u32,
    _data: u32,
    user: *mut std::ffi::c_void,
) {
    if user.is_null() {
        return;
    }

    let sync_data = unsafe { &*(user as *const SyncData) };
    let bm = Arc::clone(&sync_data.bass_mixer);
    let cs_arc = Arc::clone(&sync_data.current_stream);
    let st = Arc::clone(&sync_data.state);
    let twp = Arc::clone(&sync_data.temp_wav_path);
    let sink = sync_data.event_sink.clone();

    let old_stream = cs_arc.load(Ordering::SeqCst);
    log::info!("Track ended, playing next");

    std::thread::spawn(move || {
        let current_now = cs_arc.load(Ordering::SeqCst);
        if old_stream == 0 || current_now != old_stream {
            return;
        }

        let next_index = {
            let mut state = match st.lock() {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Failed to lock player state: {}", e);
                    return;
                }
            };
            state.get_next_index(false)
        };

        if let Some(index) = next_index {
            let (music, total_count) = {
                let state = match st.lock() {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("Failed to lock player state: {}", e);
                        return;
                    }
                };
                (state.track[index].metadata.clone(), state.track.len())
            };

            unsafe {
                BASS_Mixer_ChannelRemove(old_stream);
                BASS_StreamFree(old_stream);
                cs_arc.store(0, Ordering::SeqCst);
            }

            if MusicPlayer::load_music_inner(
                &bm,
                &cs_arc,
                &st,
                &twp,
                sink.as_ref(),
                music,
                index,
                total_count,
            ) {
                if let Ok(mut state) = st.lock() {
                    state.current_index = Some(index);
                }
                MusicPlayer::play_pause_inner(&bm, &cs_arc, true);
                MusicPlayer::emit_sync_inner(&bm, &cs_arc, &st, sink.as_deref(), true);
            }
        } else {
            std::thread::sleep(std::time::Duration::from_millis(500));

            let current_after_sleep = cs_arc.load(Ordering::SeqCst);
            if current_after_sleep != old_stream {
                return;
            }

            let first = {
                let state = match st.lock() {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("Failed to lock player state: {}", e);
                        return;
                    }
                };
                if state.track.is_empty() {
                    None
                } else {
                    Some((state.track[0].metadata.clone(), state.track.len()))
                }
            };

            if let Some((music, total_count)) = first {
                unsafe {
                    BASS_Mixer_ChannelRemove(old_stream);
                    BASS_StreamFree(old_stream);
                    cs_arc.store(0, Ordering::SeqCst);
                }

                if MusicPlayer::load_music_inner(
                    &bm,
                    &cs_arc,
                    &st,
                    &twp,
                    sink.as_ref(),
                    music,
                    0,
                    total_count,
                ) {
                    let bm_val = bm.load(Ordering::SeqCst);
                    unsafe {
                        if bm_val != 0 {
                            BASS_ChannelPause(bm_val);
                            BASS_ChannelSetPosition(bm_val, 0, BASS_POS_BYTE);
                        }
                    }

                    if let Ok(mut state) = st.lock() {
                        state.current_index = Some(0);
                    }
                    MusicPlayer::emit_sync_inner(&bm, &cs_arc, &st, sink.as_deref(), false);
                }
            } else {
                MusicPlayer::stop_stream(&bm, &cs_arc, &twp);
                if let Ok(mut state) = st.lock() {
                    state.current_index = None;
                }
            }
        }
    });
}

extern "C" fn free_sync_callback(_: u32, _: u32, _: u32, user: *mut std::ffi::c_void) {
    if !user.is_null() {
        unsafe {
            let _ = Box::from_raw(user as *mut SyncData);
        }
    }
}

unsafe impl Send for MusicPlayer {}
unsafe impl Sync for MusicPlayer {}

impl MusicPlayer {
    pub fn new(event_sink: Option<Arc<dyn EventSink>>) -> Self {
        let player = Self {
            bass_mixer: Arc::new(AtomicU32::new(0)),
            current_stream: Arc::new(AtomicU32::new(0)),
            state: Arc::new(Mutex::new(PlayerState::default())),
            temp_wav_path: Arc::new(Mutex::new(None)),
            event_sink,
        };

        player.init_bass();
        player
    }

    fn init_bass(&self) {
        unsafe {
            let mut i = 0;
            let mut info = std::mem::zeroed::<BASS_DEVICEINFO>();
            while BASS_GetDeviceInfo(i, &mut info) != 0 {
                let name = if info.name.is_null() {
                    "Unknown".to_string()
                } else {
                    std::ffi::CStr::from_ptr(info.name)
                        .to_string_lossy()
                        .into_owned()
                };
                let driver = if info.driver.is_null() {
                    "Unknown".to_string()
                } else {
                    std::ffi::CStr::from_ptr(info.driver)
                        .to_string_lossy()
                        .into_owned()
                };

                if (info.flags & 2) != 0 {
                    log::info!("Default Audio Device: {} ({})", name, driver);
                } else if (info.flags & 1) != 0 {
                    log::debug!("Available Audio Device {}: {} ({})", i, name, driver);
                }
                i += 1;
            }

            if BASS_Init(-1, 192000, 0, ptr::null_mut(), ptr::null_mut()) == 0 {
                log::error!("Failed to initialize BASS, error: {}", BASS_ErrorGetCode());
            } else {
                let mut info = std::mem::zeroed::<BASS_INFO>();
                if BASS_GetInfo(&mut info) != 0 {
                    log::info!(
                        "BASS initialized at {} Hz, Latency: {}ms, MinBuf: {}ms",
                        info.freq,
                        info.latency,
                        info.minbuf
                    );
                }
            }

            #[cfg(target_os = "macos")]
            let extension = "dylib";
            #[cfg(target_os = "windows")]
            let extension = "dll";
            #[cfg(target_os = "linux")]
            let extension = "so";

            for plugin in BASS_PLUGINS {
                #[cfg(not(target_os = "linux"))]
                let c_path = CString::new(format!("{}.{}", plugin, extension)).unwrap();
                #[cfg(target_os = "linux")]
                let c_path = CString::new(format!("lib{}.{}", plugin, extension)).unwrap();

                let handle = BASS_PluginLoad(c_path.as_ptr(), 0);
                if handle == 0 {
                    log::warn!(
                        "Failed to load plugin: {}, error: {}",
                        plugin,
                        BASS_ErrorGetCode()
                    );
                } else {
                    log::info!("Loaded plugin: {}", plugin);
                }
            }

            let mixer = BASS_Mixer_StreamCreate(44100, 2, BASS_SAMPLE_FLOAT);
            if mixer == 0 {
                log::error!(
                    "Failed to create BASS mixer stream, error: {}",
                    BASS_ErrorGetCode()
                );
            } else {
                log::info!("BASS mixer created successfully");
                self.bass_mixer.store(mixer, Ordering::SeqCst);
            }
        }
    }

    pub fn play(&self) {
        let (has_track, is_ended) = {
            let state = self.state.lock().unwrap();
            let is_ended = state.current_index.is_some()
                && state.current_index == Some(0)
                && self.current_stream.load(Ordering::SeqCst) == 0;
            (!state.track.is_empty(), is_ended)
        };

        if has_track && is_ended {
            self.goto_track(0);
        } else {
            self.play_pause(true);
        }
    }

    pub fn pause(&self) {
        self.play_pause(false);
    }

    pub fn toggle_play(&self) {
        let sync = self.get_sync_info(false);
        if sync.is_playing {
            self.pause();
        } else {
            self.play();
        }
    }

    pub fn next(&self) {
        self.play_next(true);
    }

    pub fn previous(&self) {
        self.play_previous();
    }

    pub fn clear(&self) {
        self.clear_track();
    }

    pub fn queue_count(&self) -> usize {
        self.state.lock().map(|s| s.track.len()).unwrap_or(0)
    }

    pub fn queue_get_by_index(&self, index: usize) -> Option<MusicMetadata> {
        self.state
            .lock()
            .ok()
            .and_then(|s| s.track.get(index).map(|p| p.metadata.clone()))
    }

    pub fn shuffle_track(&self) {
        if let Ok(mut state) = self.state.lock() {
            if state.original_track.is_some() {
                if let Some(original) = state.original_track.take() {
                    let current_meta = state
                        .current_index
                        .and_then(|i| state.track.get(i))
                        .map(|t| t.metadata.clone());

                    state.track = original;

                    if let Some(meta) = current_meta {
                        state.current_index =
                            state.track.iter().position(|t| t.metadata.id == meta.id);
                    }
                }
            } else {
                let len = state.track.len();
                if len > 0 {
                    state.original_track = Some(state.track.clone());
                    let mut r = rand::rng();
                    if let Some(current) = state.current_index {
                        let current_item = state.track.remove(current);
                        state.track.shuffle(&mut r);
                        state.track.insert(0, current_item);
                        state.current_index = Some(0);
                    } else {
                        state.track.shuffle(&mut r);
                    }
                }
            }
        }
        self.emit_sync(false);
    }

    pub fn set_repeat_mode(&self, mode: RepeatMode) {
        if let Ok(mut state) = self.state.lock() {
            state.repeat_mode = mode;
        }
        self.emit_sync(false);
    }

    pub fn set_pos(&self, position_ms: u64) {
        let bass_mixer = self.bass_mixer.load(Ordering::SeqCst);
        let current_stream = self.current_stream.load(Ordering::SeqCst);

        unsafe {
            if current_stream != 0 && bass_mixer != 0 {
                BASS_ChannelPause(bass_mixer);
                let seconds = position_ms as f64 / 1000.0;
                let byte_pos = BASS_ChannelSeconds2Bytes(current_stream, seconds);
                if BASS_ChannelSetPosition(current_stream, byte_pos, BASS_POS_BYTE) == 0 {
                    log::error!("Failed to set position, error: {}", BASS_ErrorGetCode());
                }
                BASS_ChannelPlay(bass_mixer, 1);
            }
        }

        self.emit_sync(false);
    }

    pub fn get_current_track(&self) -> Option<MusicMetadata> {
        let state = self.state.lock().unwrap();
        if let Some(idx) = state.current_index {
            state.track.get(idx).map(|item| item.metadata.clone())
        } else {
            None
        }
    }

    pub fn get_current_duration(&self) -> f64 {
        let current_stream = self.current_stream.load(Ordering::SeqCst);
        unsafe {
            if current_stream == 0 {
                return 0.0;
            }
            let byte_pos = BASS_ChannelGetPosition(current_stream, BASS_POS_BYTE);
            BASS_ChannelBytes2Seconds(current_stream, byte_pos) * 1000.0
        }
    }

    pub fn get_sync_info(&self, is_reset: bool) -> MusicPlayerSync {
        let bass_mixer = self.bass_mixer.load(Ordering::SeqCst);
        let current_stream = self.current_stream.load(Ordering::SeqCst);

        let current_position = if is_reset || current_stream == 0 {
            Some(0.0)
        } else {
            unsafe {
                let byte_pos = BASS_ChannelGetPosition(current_stream, BASS_POS_BYTE);
                Some(BASS_ChannelBytes2Seconds(current_stream, byte_pos))
            }
        };

        let duration = if current_stream == 0 {
            None
        } else {
            unsafe {
                let len_bytes = BASS_ChannelGetLength(current_stream, BASS_POS_BYTE);
                let sec = BASS_ChannelBytes2Seconds(current_stream, len_bytes);
                if sec > 0.0 { Some(sec) } else { None }
            }
        };

        let is_playing = if is_reset {
            true
        } else if bass_mixer == 0 {
            false
        } else {
            unsafe {
                let status = BASS_ChannelIsActive(bass_mixer);
                status == BASS_ACTIVE_PLAYING || status == BASS_ACTIVE_STALLED
            }
        };

        let (index, repeat_mode, is_shuffled) = self
            .state
            .lock()
            .map(|s| {
                (
                    s.current_index.map(|i| i as i64).unwrap_or(-1),
                    s.repeat_mode,
                    s.original_track.is_some(),
                )
            })
            .unwrap_or((-1, RepeatMode::None, false));

        MusicPlayerSync {
            index,
            current_position,
            duration,
            is_playing,
            repeat_mode,
            is_shuffled,
        }
    }

    pub fn add_track(&self, track: Vec<MusicMetadata>) {
        let was_empty = self.add_track_no_auto_play(track);
        if was_empty {
            self.goto_track(0);
        }
    }

    pub fn add_track_no_auto_play(&self, track: Vec<MusicMetadata>) -> bool {
        let mut state = match self.state.lock() {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to lock player state: {}", e);
                return false;
            }
        };
        let was_empty = state.track.is_empty();
        let mut items = Vec::with_capacity(track.len());
        for music in track {
            items.push(TrackItem { metadata: music });
        }

        if let Some(ref mut original) = state.original_track {
            original.extend(items.clone());
            let cur_idx = state.current_index.unwrap_or(0);
            if cur_idx < state.track.len() {
                let insert_pos = cur_idx + 1;
                state.track.splice(insert_pos..insert_pos, items);
                let mut rng = rand::rng();
                state.track[insert_pos..].shuffle(&mut rng);
            } else {
                state.track.extend(items);
            }
        } else {
            state.track.extend(items);
        }

        was_empty
    }

    pub fn remove_track(&self, index: usize) {
        let mut state = match self.state.lock() {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to lock player state: {}", e);
                return;
            }
        };

        if index >= state.track.len() {
            return;
        }

        let removed = state.track.remove(index);

        if let Some(ref mut original) = state.original_track {
            if let Some(orig_idx) = original
                .iter()
                .position(|t| t.metadata.id == removed.metadata.id)
            {
                original.remove(orig_idx);
            }
        }

        if let Some(current) = state.current_index {
            if current == index {
                state.current_index = None;
                drop(state);
                self.stop_current_stream();
                return;
            } else if index < current {
                state.current_index = Some(current - 1);
            }
        }
    }

    pub fn goto_track(&self, index: usize) {
        let state_arc = Arc::clone(&self.state);
        let bass_mixer = Arc::clone(&self.bass_mixer);
        let current_stream = Arc::clone(&self.current_stream);
        let temp_wav_path = Arc::clone(&self.temp_wav_path);
        let sink = self.event_sink.clone();

        std::thread::spawn(move || {
            let (music, total_count) = {
                let state = match state_arc.lock() {
                    Ok(s) => s,
                    Err(e) => {
                        crate::flog_err!("Player", "Failed to lock player state: {}", e);
                        return;
                    }
                };
                if index >= state.track.len() {
                    crate::flog_err!("Player", "index {} out of bounds (len: {})", index, state.track.len());
                    return;
                }
                (state.track[index].metadata.clone(), state.track.len())
            };

            crate::flog!("Player", "Playing track: '{}' from path: '{}'", music.title.as_deref().unwrap_or("?"), music.path);

            Self::stop_stream(&bass_mixer, &current_stream, &temp_wav_path);

            if Self::load_music_inner(
                &bass_mixer,
                &current_stream,
                &state_arc,
                &temp_wav_path,
                sink.as_ref(),
                music,
                index,
                total_count,
            ) {
                if let Ok(mut state) = state_arc.lock() {
                    state.current_index = Some(index);
                }
                Self::play_pause_inner(&bass_mixer, &current_stream, true);
                Self::emit_sync_inner(&bass_mixer, &current_stream, &state_arc, sink.as_deref(), true);
                crate::flog!("Player", "Playback started");
            } else {
                crate::flog_err!("Player", "load_music_inner failed");
            }
        });
    }

    pub fn play_next(&self, from_user: bool) {
        let state_arc = Arc::clone(&self.state);
        let bass_mixer = Arc::clone(&self.bass_mixer);
        let current_stream = Arc::clone(&self.current_stream);
        let temp_wav_path = Arc::clone(&self.temp_wav_path);
        let sink = self.event_sink.clone();

        std::thread::spawn(move || {
            let next_index = {
                let mut state = match state_arc.lock() {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("Failed to lock player state: {}", e);
                        return;
                    }
                };
                state.get_next_index(from_user)
            };

            if let Some(index) = next_index {
                let (music, total_count) = {
                    let state = match state_arc.lock() {
                        Ok(s) => s,
                        Err(e) => {
                            log::error!("Failed to lock player state: {}", e);
                            return;
                        }
                    };
                    (state.track[index].metadata.clone(), state.track.len())
                };

                let bm = bass_mixer.load(Ordering::SeqCst);
                let cs = current_stream.load(Ordering::SeqCst);
                unsafe {
                    if cs != 0 {
                        BASS_Mixer_ChannelRemove(cs);
                        BASS_StreamFree(cs);
                        current_stream.store(0, Ordering::SeqCst);
                    }
                    if from_user && bm != 0 {
                        BASS_ChannelSetPosition(bm, 0, BASS_POS_BYTE);
                    }
                }

                if Self::load_music_inner(
                    &bass_mixer,
                    &current_stream,
                    &state_arc,
                    &temp_wav_path,
                    sink.as_ref(),
                    music,
                    index,
                    total_count,
                ) {
                    if let Ok(mut state) = state_arc.lock() {
                        state.current_index = Some(index);
                    }
                    Self::play_pause_inner(&bass_mixer, &current_stream, true);
                    Self::emit_sync_inner(
                        &bass_mixer,
                        &current_stream,
                        &state_arc,
                        sink.as_deref(),
                        true,
                    );
                }
            } else if !from_user {
                let first = {
                    let state = match state_arc.lock() {
                        Ok(s) => s,
                        Err(e) => {
                            log::error!("Failed to lock player state: {}", e);
                            return;
                        }
                    };
                    if state.track.is_empty() {
                        None
                    } else {
                        Some((state.track[0].metadata.clone(), state.track.len()))
                    }
                };

                if let Some((music, total_count)) = first {
                    let cs = current_stream.load(Ordering::SeqCst);
                    unsafe {
                        if cs != 0 {
                            BASS_Mixer_ChannelRemove(cs);
                            BASS_StreamFree(cs);
                            current_stream.store(0, Ordering::SeqCst);
                        }
                    }

                    if Self::load_music_inner(
                        &bass_mixer,
                        &current_stream,
                        &state_arc,
                        &temp_wav_path,
                        sink.as_ref(),
                        music,
                        0,
                        total_count,
                    ) {
                        let bm = bass_mixer.load(Ordering::SeqCst);
                        unsafe {
                            if bm != 0 {
                                BASS_ChannelPause(bm);
                                BASS_ChannelSetPosition(bm, 0, BASS_POS_BYTE);
                            }
                        }

                        if let Ok(mut state) = state_arc.lock() {
                            state.current_index = Some(0);
                        }
                        Self::emit_sync_inner(
                            &bass_mixer,
                            &current_stream,
                            &state_arc,
                            sink.as_deref(),
                            false,
                        );
                    }
                } else {
                    Self::stop_stream(&bass_mixer, &current_stream, &temp_wav_path);
                    if let Ok(mut state) = state_arc.lock() {
                        state.current_index = None;
                    }
                }
            } else {
                Self::stop_stream(&bass_mixer, &current_stream, &temp_wav_path);
                if let Ok(mut state) = state_arc.lock() {
                    state.current_index = Some(0);
                }
                Self::emit_sync_inner(
                    &bass_mixer,
                    &current_stream,
                    &state_arc,
                    sink.as_deref(),
                    false,
                );
            }
        });
    }

    pub fn play_previous(&self) {
        let state_arc = Arc::clone(&self.state);
        let bass_mixer = Arc::clone(&self.bass_mixer);
        let current_stream = Arc::clone(&self.current_stream);
        let temp_wav_path = Arc::clone(&self.temp_wav_path);
        let sink = self.event_sink.clone();

        std::thread::spawn(move || {
            let prev_index = {
                let state = match state_arc.lock() {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("Failed to lock player state: {}", e);
                        return;
                    }
                };
                match state.current_index {
                    Some(current) => {
                        if current == 0 && state.repeat_mode == RepeatMode::None {
                            Some(0)
                        } else if current > 0 {
                            Some(current - 1)
                        } else if !state.track.is_empty() {
                            Some(state.track.len() - 1)
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            };

            if let Some(index) = prev_index {
                let (music, total_count) = {
                    let state = match state_arc.lock() {
                        Ok(s) => s,
                        Err(e) => {
                            log::error!("Failed to lock player state: {}", e);
                            return;
                        }
                    };
                    (state.track[index].metadata.clone(), state.track.len())
                };

                Self::stop_stream(&bass_mixer, &current_stream, &temp_wav_path);

                if Self::load_music_inner(
                    &bass_mixer,
                    &current_stream,
                    &state_arc,
                    &temp_wav_path,
                    sink.as_ref(),
                    music,
                    index,
                    total_count,
                ) {
                    if let Ok(mut state) = state_arc.lock() {
                        state.current_index = Some(index);
                    }
                    Self::play_pause_inner(&bass_mixer, &current_stream, true);
                    Self::emit_sync_inner(
                        &bass_mixer,
                        &current_stream,
                        &state_arc,
                        sink.as_deref(),
                        true,
                    );
                }
            }
        });
    }

    pub fn moveto_track(&self, from: usize, to: usize) {
        {
            let mut state = match self.state.lock() {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Failed to lock player state: {}", e);
                    return;
                }
            };

            if from >= state.track.len() || to >= state.track.len() {
                return;
            }

            let item = state.track.remove(from);
            state.track.insert(to, item);
            state.original_track = None;

            if let Some(current) = state.current_index {
                state.current_index = Some(if current == from {
                    to
                } else if from < current && to >= current {
                    current - 1
                } else if from > current && to <= current {
                    current + 1
                } else {
                    current
                });
            }
        }
        self.emit_sync(false);
    }

    pub fn set_volume(&self, volume: f32) {
        let bass_mixer = self.bass_mixer.load(Ordering::SeqCst);
        let clamped = volume.clamp(0.0, 2.0);

        unsafe {
            if bass_mixer != 0
                && BASS_ChannelSetAttribute(bass_mixer, BASS_ATTRIB_VOL, clamped) == 0
            {
                log::error!("Failed to set volume, error: {}", BASS_ErrorGetCode());
            }
        }
    }

    pub fn emit_sync(&self, is_reset: bool) {
        Self::emit_sync_inner(
            &self.bass_mixer,
            &self.current_stream,
            &self.state,
            self.event_sink.as_deref(),
            is_reset,
        );
    }

    fn play_pause(&self, play: bool) {
        Self::play_pause_inner(&self.bass_mixer, &self.current_stream, play);
    }

    fn play_pause_inner(bass_mixer: &Arc<AtomicU32>, _current_stream: &Arc<AtomicU32>, play: bool) {
        let bm = bass_mixer.load(Ordering::SeqCst);
        unsafe {
            if bm == 0 {
                return;
            }
            if play {
                if BASS_ChannelPlay(bm, 0) == 0 {
                    log::error!("Failed to play, error: {}", BASS_ErrorGetCode());
                }
            } else if BASS_ChannelPause(bm) == 0 {
                log::error!("Failed to pause, error: {}", BASS_ErrorGetCode());
            }
        }
    }

    fn clear_track(&self) {
        let bm = self.bass_mixer.load(Ordering::SeqCst);
        unsafe {
            if bm != 0 {
                BASS_ChannelStop(bm);
                BASS_ChannelSetPosition(bm, 0, BASS_POS_BYTE);
            }
        }

        self.stop_current_stream();
        if let Ok(mut state) = self.state.lock() {
            state.track.clear();
            state.original_track = None;
            state.current_index = None;
        }
    }

    fn stop_current_stream(&self) {
        Self::stop_stream(&self.bass_mixer, &self.current_stream, &self.temp_wav_path);
    }

    fn stop_stream(
        bass_mixer: &Arc<AtomicU32>,
        current_stream: &Arc<AtomicU32>,
        temp_wav_path: &Arc<Mutex<Option<PathBuf>>>,
    ) {
        Self::cleanup_temp_wav_inner(temp_wav_path);

        let bm = bass_mixer.load(Ordering::SeqCst);
        let cs = current_stream.load(Ordering::SeqCst);

        unsafe {
            if cs != 0 {
                BASS_ChannelStop(cs);
                BASS_Mixer_ChannelRemove(cs);
                BASS_StreamFree(cs);
                current_stream.store(0, Ordering::SeqCst);
            }
            if bm != 0 {
                BASS_ChannelSetPosition(bm, 0, BASS_POS_BYTE);
            }
        }
    }

    fn cleanup_temp_wav_inner(temp_wav_path: &Arc<Mutex<Option<PathBuf>>>) {
        if let Ok(mut guard) = temp_wav_path.lock() {
            if let Some(path) = guard.take() {
                if path.exists() {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }

    fn emit_sync_inner(
        bass_mixer: &Arc<AtomicU32>,
        current_stream: &Arc<AtomicU32>,
        state: &Arc<Mutex<PlayerState>>,
        sink: Option<&dyn EventSink>,
        is_reset: bool,
    ) {
        let bm = bass_mixer.load(Ordering::SeqCst);
        let cs = current_stream.load(Ordering::SeqCst);

        let current_position = if is_reset || cs == 0 {
            Some(0.0)
        } else {
            unsafe {
                let byte_pos = BASS_ChannelGetPosition(cs, BASS_POS_BYTE);
                Some(BASS_ChannelBytes2Seconds(cs, byte_pos))
            }
        };

        let duration = if cs == 0 {
            None
        } else {
            unsafe {
                let len_bytes = BASS_ChannelGetLength(cs, BASS_POS_BYTE);
                let sec = BASS_ChannelBytes2Seconds(cs, len_bytes);
                if sec > 0.0 { Some(sec) } else { None }
            }
        };

        let is_playing = if is_reset {
            true
        } else if bm == 0 {
            false
        } else {
            unsafe {
                let status = BASS_ChannelIsActive(bm);
                status == BASS_ACTIVE_PLAYING || status == BASS_ACTIVE_STALLED
            }
        };

        let (index, repeat_mode, is_shuffled) = state
            .lock()
            .map(|s| {
                (
                    s.current_index.map(|i| i as i64).unwrap_or(-1),
                    s.repeat_mode,
                    s.original_track.is_some(),
                )
            })
            .unwrap_or((-1, RepeatMode::None, false));

        let sync_state = MusicPlayerSync {
            index,
            current_position,
            duration,
            is_playing,
            repeat_mode,
            is_shuffled,
        };

        if let Some(sink) = sink {
            sink.on_player_sync(sync_state);
        }
    }

    fn setup_sync(
        stream: u32,
        bass_mixer: &Arc<AtomicU32>,
        current_stream: &Arc<AtomicU32>,
        state: &Arc<Mutex<PlayerState>>,
        temp_wav_path: &Arc<Mutex<Option<PathBuf>>>,
        event_sink: Option<&Arc<dyn EventSink>>,
    ) {
        if stream == 0 {
            return;
        }

        let sync_data = Box::into_raw(Box::new(SyncData {
            bass_mixer: Arc::clone(bass_mixer),
            current_stream: Arc::clone(current_stream),
            state: Arc::clone(state),
            temp_wav_path: Arc::clone(temp_wav_path),
            event_sink: event_sink.cloned(),
        }));

        unsafe {
            BASS_ChannelSetSync(
                stream,
                BASS_SYNC_END | BASS_SYNC_MIXTIME | 0x80000000,
                0,
                Some(end_sync_callback),
                sync_data as *mut _,
            );
            BASS_ChannelSetSync(
                stream,
                BASS_SYNC_FREE | 0x80000000,
                0,
                Some(free_sync_callback),
                sync_data as *mut _,
            );
        }
    }

    fn load_music_inner(
        bass_mixer: &Arc<AtomicU32>,
        current_stream: &Arc<AtomicU32>,
        state: &Arc<Mutex<PlayerState>>,
        temp_wav_path: &Arc<Mutex<Option<PathBuf>>>,
        sink: Option<&Arc<dyn EventSink>>,
        music: MusicMetadata,
        index: usize,
        _total_count: usize,
    ) -> bool {
        let mixer = bass_mixer.load(Ordering::SeqCst);
        if mixer == 0 {
            log::error!("Mixer is not initialized");
            return false;
        }

        // On macOS/Linux, BASS_StreamCreateFile expects UTF-8 char* without BASS_UNICODE.
        // BASS_UNICODE instructs BASS on Windows to expect UTF-16 wchar_t*.
        #[cfg(target_os = "windows")]
        let (flags, path_ptr) = {
            use std::os::windows::ffi::OsStrExt;
            let wide: Vec<u16> = std::ffi::OsStr::new(&music.path)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            (BASS_STREAM_DECODE | BASS_SAMPLE_FLOAT | BASS_UNICODE, wide.as_ptr() as *const std::ffi::c_void)
        };

        #[cfg(not(target_os = "windows"))]
        let (flags, path_ptr) = {
            let c_path = match CString::new(music.path.as_str()) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("[Player] Invalid path string: {}", e);
                    return false;
                }
            };
            eprintln!("[Player] Calling BASS_StreamCreateFile with path: '{}'", music.path);
            (BASS_STREAM_DECODE | BASS_SAMPLE_FLOAT, c_path.into_raw() as *const std::ffi::c_void)
        };

        let stream = unsafe {
            BASS_StreamCreateFile(
                0,
                path_ptr,
                0,
                0,
                flags,
            )
        };

        #[cfg(not(target_os = "windows"))]
        unsafe {
            // Reclaim CString memory after BASS creates stream
            drop(CString::from_raw(path_ptr as *mut std::ffi::c_char));
        };

        if stream == 0 {
            let err = unsafe { BASS_ErrorGetCode() };
            crate::flog_err!(
                "Player",
                "Failed to load music file {}: BASS error {}",
                music.path,
                err
            );
            return false;
        }

        Self::setup_sync(
            stream,
            bass_mixer,
            current_stream,
            state,
            temp_wav_path,
            sink,
        );

        if unsafe { BASS_Mixer_StreamAddChannel(mixer, stream, BASS_MIXER_NORAMPIN) } == 0 {
            log::error!(
                "Failed to add stream to mixer: {}",
                unsafe { BASS_ErrorGetCode() }
            );
            unsafe { BASS_StreamFree(stream) };
            return false;
        }

        current_stream.store(stream, Ordering::SeqCst);

        if let Some(sink_ref) = sink {
            sink_ref.on_track_changed(Some(music), index);
        }

        true
    }
}

impl Drop for MusicPlayer {
    fn drop(&mut self) {
        unsafe {
            BASS_Free();
        }
    }
}
