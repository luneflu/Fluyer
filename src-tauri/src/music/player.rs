use crate::music::metadata::MusicMetadata;
use crate::state::{app_handle, main_window};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::path::PathBuf;
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{Emitter, Manager};
#[cfg(target_os = "android")]
use tauri_plugin_fluyer::FluyerExt;

use super::bass::*;

#[derive(Clone, Debug)]
struct TrackItem {
    metadata: MusicMetadata,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RepeatMode {
    #[serde(rename = "repeatNone")]
    None,
    #[serde(rename = "repeat")]
    All,
    #[serde(rename = "repeatOne")]
    One,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicPlayerSync {
    index: i64,
    current_position: Option<f64>,
    is_playing: bool,
    repeat_mode: RepeatMode,
    is_shuffled: bool,
}

impl MusicPlayerSync {
    pub(crate) fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// Current playback position in milliseconds.
    pub(crate) fn position_ms(&self) -> u64 {
        self.current_position
            .map(|s| (s * 1000.0) as u64)
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone)]
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
}

struct SyncData {
    bass_mixer: Arc<AtomicU32>,
    current_stream: Arc<AtomicU32>,
    state: Arc<Mutex<PlayerState>>,
    temp_wav_path: Arc<Mutex<Option<PathBuf>>>,
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

    let old_stream = cs_arc.load(Ordering::SeqCst);
    crate::info!("Track ended, playing next");

    tauri::async_runtime::spawn_blocking(move || {
        let current_now = cs_arc.load(Ordering::SeqCst);
        if old_stream == 0 || current_now != old_stream {
            return;
        }

        let next_index = {
            let mut state = match st.lock() {
                Ok(s) => s,
                Err(e) => {
                    crate::error!("Failed to lock player state: {}", e);
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
                        crate::error!("Failed to lock player state: {}", e);
                        return;
                    }
                };
                (state.track[index].metadata.clone(), state.track.len())
            };

            #[cfg(desktop)]
            unsafe {
                BASS_Mixer_ChannelRemove(old_stream);
                BASS_StreamFree(old_stream);
                cs_arc.store(0, Ordering::SeqCst);
            }
            #[cfg(target_os = "android")]
            if let Some(bass) = bass_android::get_bass() {
                unsafe {
                    (bass.bass_mixer_channel_remove)(old_stream);
                    (bass.bass_stream_free)(old_stream);
                    cs_arc.store(0, Ordering::SeqCst);
                }
            }

            if MusicPlayer::load_music_inner(&bm, &cs_arc, &st, &twp, music, index, total_count) {
                // Update current_index BEFORE resuming the mixer so that if the
                // new stream's END sync fires immediately (very short track), it
                // sees the correct index when calling get_next_index.
                if let Ok(mut state) = st.lock() {
                    state.current_index = Some(index);
                }
                // Resume the mixer in case it stopped while the old stream was
                // removed and the next one was being loaded (buffer drain gap).
                // No-op when the mixer is still running, so gapless keeps working.
                MusicPlayer::play_pause_inner(&bm, &cs_arc, true);
                MusicPlayer::emit_sync_inner(&bm, &cs_arc, &st, true);
            }
        } else {
            // Queue is at last track
            // Wait for the tail of the current track to finish playing
            // from the mixer's output buffer (default BASS buffer is 500ms).
            // This prevents the end of the last track from being abruptly cut off.
            std::thread::sleep(std::time::Duration::from_millis(500));

            // Verify the user hasn't started playing something else during the sleep
            let current_after_sleep = cs_arc.load(Ordering::SeqCst);
            if current_after_sleep != old_stream {
                return;
            }

            let first = {
                let state = match st.lock() {
                    Ok(s) => s,
                    Err(e) => {
                        crate::error!("Failed to lock player state: {}", e);
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
                #[cfg(desktop)]
                unsafe {
                    BASS_Mixer_ChannelRemove(old_stream);
                    BASS_StreamFree(old_stream);
                    cs_arc.store(0, Ordering::SeqCst);
                }
                #[cfg(target_os = "android")]
                if let Some(bass) = bass_android::get_bass() {
                    unsafe {
                        (bass.bass_mixer_channel_remove)(old_stream);
                        (bass.bass_stream_free)(old_stream);
                        cs_arc.store(0, Ordering::SeqCst);
                    }
                }

                if MusicPlayer::load_music_inner(&bm, &cs_arc, &st, &twp, music, 0, total_count) {
                    let bm_val = bm.load(Ordering::SeqCst);
                    #[cfg(desktop)]
                    unsafe {
                        if bm_val != 0 {
                            BASS_ChannelPause(bm_val);
                            BASS_ChannelSetPosition(bm_val, 0, BASS_POS_BYTE);
                        }
                    }
                    #[cfg(target_os = "android")]
                    if let Some(bass) = bass_android::get_bass() {
                        unsafe {
                            if bm_val != 0 {
                                (bass.bass_channel_pause)(bm_val);
                                (bass.bass_channel_set_position)(bm_val, 0, BASS_POS_BYTE);
                            }
                        }
                    }

                    if let Ok(mut state) = st.lock() {
                        state.current_index = Some(0);
                    }
                    MusicPlayer::emit_sync_inner(&bm, &cs_arc, &st, false);
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

// BASS handles are plain u32 values – not OS handles tied to a specific thread –
// so it is safe to send/share them across threads as long as we serialise access
// ourselves (AtomicU32 / Mutex already do that).
unsafe impl Send for MusicPlayer {}
unsafe impl Sync for MusicPlayer {}

impl MusicPlayer {
    pub fn spawn() -> Self {
        let player = Self {
            bass_mixer: Arc::new(AtomicU32::new(0)),
            current_stream: Arc::new(AtomicU32::new(0)),
            state: Arc::new(Mutex::new(PlayerState {
                track: Vec::new(),
                original_track: None,
                current_index: None,
                repeat_mode: RepeatMode::None,
            })),
            temp_wav_path: Arc::new(Mutex::new(None)),
        };

        player.start_focus_listener();
        player.init_bass();

        #[cfg(target_os = "android")]
        {
            crate::music::media_session::MediaSession::init();
        }

        player
    }

    fn init_bass(&self) {
        #[cfg(desktop)]
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
                    // BASS_DEVICE_DEFAULT
                    crate::info!("Default Audio Device: {} ({})", name, driver);
                } else if (info.flags & 1) != 0 {
                    // BASS_DEVICE_ENABLED
                    crate::debug!("Available Audio Device {}: {} ({})", i, name, driver);
                }
                i += 1;
            }

            if BASS_Init(-1, 192000, 0, ptr::null_mut(), ptr::null_mut()) == 0 {
                crate::error!("Failed to initialize BASS, error: {}", BASS_ErrorGetCode());
            } else {
                let mut info = std::mem::zeroed::<BASS_INFO>();
                if BASS_GetInfo(&mut info) != 0 {
                    crate::info!(
                        "BASS initialized successfully at {} Hz, Latency: {}ms, MinBuf: {}ms",
                        info.freq,
                        info.latency,
                        info.minbuf
                    );
                } else {
                    crate::info!("BASS initialized successfully");
                }
            }

            // Load plugins based on platform
            #[cfg(target_os = "macos")]
            let extension = "dylib";
            #[cfg(target_os = "windows")]
            let extension = "dll";
            #[cfg(target_os = "linux")]
            let extension = "so";

            for plugin in BASS_PLUGINS {
                #[cfg(target_os = "macos")]
                if plugin == "bassalac" || plugin == "bass_aac" {
                    continue;
                }

                #[cfg(not(target_os = "linux"))]
                let c_path = CString::new(format!("{}.{}", plugin, extension)).unwrap();
                #[cfg(target_os = "linux")]
                let c_path = CString::new(format!("lib{}.{}", plugin, extension)).unwrap();

                let handle = BASS_PluginLoad(c_path.as_ptr(), 0);
                if handle == 0 {
                    crate::warn!(
                        "Failed to load plugin: {}, error: {}",
                        plugin,
                        BASS_ErrorGetCode()
                    );
                } else {
                    crate::info!("Loaded plugin: {}", plugin);
                }
            }

            let mixer = BASS_Mixer_StreamCreate(44100, 2, BASS_SAMPLE_FLOAT | BASS_MIXER_NONSTOP);
            if mixer == 0 {
                crate::error!(
                    "Failed to create BASS mixer stream, error: {}",
                    BASS_ErrorGetCode()
                );
            } else {
                crate::info!("BASS mixer created successfully");
                self.bass_mixer.store(mixer, Ordering::SeqCst);
            }
        }

        #[cfg(target_os = "android")]
        {
            if let Err(e) = bass_android::initialize_bass() {
                crate::error!("Failed to initialize BASS on Android: {}", e);
            } else {
                crate::info!("BASS libraries loaded successfully on Android");

                if let Some(bass) = bass_android::get_bass() {
                    unsafe {
                        if (bass.bass_init)(-1, 44100, 0, ptr::null_mut(), ptr::null_mut()) == 0 {
                            crate::error!(
                                "Failed to initialize BASS, error: {}",
                                (bass.bass_error_get_code)()
                            );
                        } else {
                            crate::info!("BASS initialized successfully");
                        }

                        for plugin in BASS_PLUGINS {
                            let lib_name = format!("lib{}.so", plugin);
                            let c_path = CString::new(lib_name).unwrap();
                            let handle = (bass.bass_plugin_load)(c_path.as_ptr() as *const i8, 0);
                            if handle == 0 {
                                crate::warn!(
                                    "Failed to load {} plugin, error: {}",
                                    plugin,
                                    (bass.bass_error_get_code)()
                                );
                            } else {
                                crate::info!("Loaded {} plugin", plugin);
                            }
                        }

                        let mixer = (bass.bass_mixer_stream_create)(
                            44100,
                            2,
                            BASS_SAMPLE_FLOAT | BASS_MIXER_NONSTOP,
                        );
                        if mixer == 0 {
                            crate::error!(
                                "Failed to create BASS mixer stream, error: {}",
                                (bass.bass_error_get_code)()
                            );
                        } else {
                            crate::info!("BASS mixer created successfully");
                            self.bass_mixer.store(mixer, Ordering::SeqCst);
                        }
                    }
                }
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
        use rand::rng;
        use rand::seq::SliceRandom;

        if let Ok(mut state) = self.state.lock() {
            if state.original_track.is_some() {
                // Disable shuffle: restore original track list
                if let Some(original) = state.original_track.take() {
                    let current_meta = state
                        .current_index
                        .and_then(|i| state.track.get(i))
                        .map(|t| t.metadata.clone());

                    state.track = original;

                    if let Some(meta) = current_meta {
                        // Restore current index relative to original list
                        state.current_index =
                            state.track.iter().position(|t| t.metadata.id == meta.id);
                    }
                }
            } else {
                // Enable shuffle
                let len = state.track.len();
                if len > 0 {
                    state.original_track = Some(state.track.clone());

                    let mut r = rng();
                    if let Some(current) = state.current_index {
                        // Extract current item and keep it at index 0 (or original pos)
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

        #[cfg(target_os = "android")]
        {
            let (current_idx, count) = if let Ok(state) = self.state.lock() {
                (state.current_index, state.track.len())
            } else {
                (None, 0)
            };
            self.update_android_media_boundaries(current_idx, count);
        }
    }

    pub fn set_pos(&self, position: u64) {
        let bass_mixer = self.bass_mixer.load(Ordering::SeqCst);
        let current_stream = self.current_stream.load(Ordering::SeqCst);

        #[cfg(desktop)]
        unsafe {
            if current_stream != 0 && bass_mixer != 0 {
                BASS_ChannelPause(bass_mixer);
                let seconds = position as f64 / 1000.0;
                let byte_pos = BASS_ChannelSeconds2Bytes(current_stream, seconds);
                if BASS_ChannelSetPosition(current_stream, byte_pos, BASS_POS_BYTE) == 0 {
                    crate::error!("Failed to set position, error: {}", BASS_ErrorGetCode());
                }
                BASS_ChannelPlay(bass_mixer, 1);
            }
        }

        #[cfg(target_os = "android")]
        {
            if let Some(bass) = bass_android::get_bass() {
                unsafe {
                    if current_stream != 0 && bass_mixer != 0 {
                        (bass.bass_channel_pause)(bass_mixer);
                        let seconds = position as f64 / 1000.0;
                        let byte_pos = (bass.bass_channel_seconds2bytes)(current_stream, seconds);
                        if (bass.bass_channel_set_position)(current_stream, byte_pos, BASS_POS_BYTE)
                            == 0
                        {
                            crate::error!(
                                "Failed to set position, error: {}",
                                (bass.bass_error_get_code)()
                            );
                        }
                        (bass.bass_channel_play)(bass_mixer, 1);
                    }
                }
            }
        }

        let sync_info = self.get_sync_info(false);
        crate::music::media_session::MediaSession::set_state(sync_info.is_playing, position);
        self.emit_sync(false);
    }

    pub fn get_current_duration(&self) -> f64 {
        let current_stream = self.current_stream.load(Ordering::SeqCst);

        #[cfg(desktop)]
        unsafe {
            if current_stream == 0 {
                return 0.0;
            }
            let byte_pos = BASS_ChannelGetPosition(current_stream, BASS_POS_BYTE);
            return BASS_ChannelBytes2Seconds(current_stream, byte_pos) * 1000.0;
        }

        #[cfg(target_os = "android")]
        {
            if let Some(bass) = bass_android::get_bass() {
                unsafe {
                    if current_stream == 0 {
                        return 0.0;
                    }
                    let byte_pos = (bass.bass_channel_get_position)(current_stream, BASS_POS_BYTE);
                    return (bass.bass_channel_bytes2seconds)(current_stream, byte_pos) * 1000.0;
                }
            }
            0.0
        }
    }

    pub fn get_sync_info(&self, is_reset: bool) -> MusicPlayerSync {
        let bass_mixer = self.bass_mixer.load(Ordering::SeqCst);
        let current_stream = self.current_stream.load(Ordering::SeqCst);

        let current_position = if is_reset || current_stream == 0 {
            Some(0.0)
        } else {
            #[cfg(desktop)]
            unsafe {
                let byte_pos = BASS_ChannelGetPosition(current_stream, BASS_POS_BYTE);
                Some(BASS_ChannelBytes2Seconds(current_stream, byte_pos) * 1000.0)
            }
            #[cfg(target_os = "android")]
            {
                bass_android::get_bass()
                    .map(|bass| unsafe {
                        let byte_pos =
                            (bass.bass_channel_get_position)(current_stream, BASS_POS_BYTE);
                        (bass.bass_channel_bytes2seconds)(current_stream, byte_pos) * 1000.0
                    })
                    .or(Some(0.0))
            }
        };

        let is_playing = if is_reset {
            true
        } else if bass_mixer == 0 {
            false
        } else {
            #[cfg(desktop)]
            unsafe {
                BASS_ChannelIsActive(bass_mixer) == BASS_ACTIVE_PLAYING
            }
            #[cfg(target_os = "android")]
            {
                bass_android::get_bass()
                    .map(|bass| unsafe {
                        (bass.bass_channel_is_active)(bass_mixer) == BASS_ACTIVE_PLAYING
                    })
                    .unwrap_or(false)
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
            is_playing,
            repeat_mode,
            is_shuffled,
        }
    }

    pub fn add_track(&self, track: Vec<MusicMetadata>) {
        let was_empty;
        {
            let mut state = match self.state.lock() {
                Ok(s) => s,
                Err(e) => {
                    crate::error!("Failed to lock player state: {}", e);
                    return;
                }
            };
            was_empty = state.track.is_empty();
            let mut items = Vec::with_capacity(track.len());
            for music in track {
                items.push(TrackItem { metadata: music });
            }

            if let Some(ref mut original) = state.original_track {
                original.extend(items.clone());
                // In shuffle mode, insert new items right after current track, then shuffle the remainder.
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
        }

        if was_empty {
            self.goto_track(0);
        } else {
            #[cfg(target_os = "android")]
            {
                let (current_index, total_count) = self
                    .state
                    .lock()
                    .map(|s| (s.current_index, s.track.len()))
                    .unwrap_or((None, 0));
                self.update_android_media_boundaries(current_index, total_count);
            }
        }
    }

    pub fn remove_track(&self, index: usize) {
        let mut state = match self.state.lock() {
            Ok(s) => s,
            Err(e) => {
                crate::error!("Failed to lock player state: {}", e);
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

        #[cfg(target_os = "android")]
        {
            let current_index = state.current_index;
            let total_count = state.track.len();
            drop(state);
            self.update_android_media_boundaries(current_index, total_count);
        }
    }

    pub fn goto_track(&self, index: usize) {
        let state_arc = Arc::clone(&self.state);
        let bass_mixer = Arc::clone(&self.bass_mixer);
        let current_stream = Arc::clone(&self.current_stream);
        let temp_wav_path = Arc::clone(&self.temp_wav_path);

        tauri::async_runtime::spawn_blocking(move || {
            let (music, total_count) = {
                let state = match state_arc.lock() {
                    Ok(s) => s,
                    Err(e) => {
                        crate::error!("Failed to lock player state: {}", e);
                        return;
                    }
                };
                if index >= state.track.len() {
                    return;
                }
                (state.track[index].metadata.clone(), state.track.len())
            };

            Self::stop_stream(&bass_mixer, &current_stream, &temp_wav_path);

            if Self::load_music_inner(
                &bass_mixer,
                &current_stream,
                &state_arc,
                &temp_wav_path,
                music,
                index,
                total_count,
            ) {
                if let Ok(mut state) = state_arc.lock() {
                    state.current_index = Some(index);
                }
                Self::play_pause_inner(&bass_mixer, &current_stream, true);
                Self::emit_sync_inner(&bass_mixer, &current_stream, &state_arc, true);
            }
        });
    }

    pub fn play_next(&self, from_user: bool) {
        let state_arc = Arc::clone(&self.state);
        let bass_mixer = Arc::clone(&self.bass_mixer);
        let current_stream = Arc::clone(&self.current_stream);
        let temp_wav_path = Arc::clone(&self.temp_wav_path);

        tauri::async_runtime::spawn_blocking(move || {
            let next_index = {
                let mut state = match state_arc.lock() {
                    Ok(s) => s,
                    Err(e) => {
                        crate::error!("Failed to lock player state: {}", e);
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
                            crate::error!("Failed to lock player state: {}", e);
                            return;
                        }
                    };
                    (state.track[index].metadata.clone(), state.track.len())
                };

                // Remove old stream from mixer. Flush buffer only on user-initiated skip
                // to preserve gapless auto-advance.
                let bm = bass_mixer.load(Ordering::SeqCst);
                let cs = current_stream.load(Ordering::SeqCst);
                #[cfg(desktop)]
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
                #[cfg(target_os = "android")]
                if let Some(bass) = bass_android::get_bass() {
                    unsafe {
                        if cs != 0 {
                            (bass.bass_mixer_channel_remove)(cs);
                            (bass.bass_stream_free)(cs);
                            current_stream.store(0, Ordering::SeqCst);
                        }
                        if from_user && bm != 0 {
                            (bass.bass_channel_set_position)(bm, 0, BASS_POS_BYTE);
                        }
                    }
                }

                if Self::load_music_inner(
                    &bass_mixer,
                    &current_stream,
                    &state_arc,
                    &temp_wav_path,
                    music,
                    index,
                    total_count,
                ) {
                    if let Ok(mut state) = state_arc.lock() {
                        state.current_index = Some(index);
                    }
                    Self::play_pause_inner(&bass_mixer, &current_stream, true);
                    Self::emit_sync_inner(&bass_mixer, &current_stream, &state_arc, true);
                }
            } else if !from_user {
                // Queue ended: reset to first track, paused
                let first = {
                    let state = match state_arc.lock() {
                        Ok(s) => s,
                        Err(e) => {
                            crate::error!("Failed to lock player state: {}", e);
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
                    #[cfg(desktop)]
                    unsafe {
                        if cs != 0 {
                            BASS_Mixer_ChannelRemove(cs);
                            BASS_StreamFree(cs);
                            current_stream.store(0, Ordering::SeqCst);
                        }
                    }
                    #[cfg(target_os = "android")]
                    if let Some(bass) = bass_android::get_bass() {
                        unsafe {
                            if cs != 0 {
                                (bass.bass_mixer_channel_remove)(cs);
                                (bass.bass_stream_free)(cs);
                                current_stream.store(0, Ordering::SeqCst);
                            }
                        }
                    }

                    if Self::load_music_inner(
                        &bass_mixer,
                        &current_stream,
                        &state_arc,
                        &temp_wav_path,
                        music,
                        0,
                        total_count,
                    ) {
                        let bm = bass_mixer.load(Ordering::SeqCst);
                        #[cfg(desktop)]
                        unsafe {
                            if bm != 0 {
                                BASS_ChannelPause(bm);
                                BASS_ChannelSetPosition(bm, 0, BASS_POS_BYTE);
                            }
                        }
                        #[cfg(target_os = "android")]
                        if let Some(bass) = bass_android::get_bass() {
                            unsafe {
                                if bm != 0 {
                                    (bass.bass_channel_pause)(bm);
                                    (bass.bass_channel_set_position)(bm, 0, BASS_POS_BYTE);
                                }
                            }
                        }

                        if let Ok(mut state) = state_arc.lock() {
                            state.current_index = Some(0);
                        }
                        Self::emit_sync_inner(&bass_mixer, &current_stream, &state_arc, false);
                    }
                } else {
                    Self::stop_stream(&bass_mixer, &current_stream, &temp_wav_path);
                    if let Ok(mut state) = state_arc.lock() {
                        state.current_index = None;
                    }
                }
            } else {
                // User pressed next at the last track: end playback
                Self::stop_stream(&bass_mixer, &current_stream, &temp_wav_path);
                if let Ok(mut state) = state_arc.lock() {
                    state.current_index = Some(0);
                }
                Self::emit_sync_inner(&bass_mixer, &current_stream, &state_arc, false);
            }
        });
    }

    pub fn play_previous(&self) {
        let state_arc = Arc::clone(&self.state);
        let bass_mixer = Arc::clone(&self.bass_mixer);
        let current_stream = Arc::clone(&self.current_stream);
        let temp_wav_path = Arc::clone(&self.temp_wav_path);

        tauri::async_runtime::spawn_blocking(move || {
            let prev_index = {
                let state = match state_arc.lock() {
                    Ok(s) => s,
                    Err(e) => {
                        crate::error!("Failed to lock player state: {}", e);
                        return;
                    }
                };
                match state.current_index {
                    Some(current) => {
                        crate::info!("Current index: {:?}", current);
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

            crate::info!("Previous Index: {:?}", prev_index);
            if let Some(index) = prev_index {
                let (music, total_count) = {
                    let state = match state_arc.lock() {
                        Ok(s) => s,
                        Err(e) => {
                            crate::error!("Failed to lock player state: {}", e);
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
                    music,
                    index,
                    total_count,
                ) {
                    if let Ok(mut state) = state_arc.lock() {
                        state.current_index = Some(index);
                    }
                    Self::play_pause_inner(&bass_mixer, &current_stream, true);
                    Self::emit_sync_inner(&bass_mixer, &current_stream, &state_arc, true);
                }
            }
        });
    }

    pub fn moveto_track(&self, from: usize, to: usize) {
        {
            let mut state = match self.state.lock() {
                Ok(s) => s,
                Err(e) => {
                    crate::error!("Failed to lock player state: {}", e);
                    return;
                }
            };

            if from >= state.track.len() || to >= state.track.len() {
                return;
            }

            let item = state.track.remove(from);
            state.track.insert(to, item.clone());

            // Disable shuffle when user manually reorders
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
        let clamped = volume.clamp(0.0, 1.0);

        #[cfg(desktop)]
        unsafe {
            if bass_mixer != 0
                && BASS_ChannelSetAttribute(bass_mixer, BASS_ATTRIB_VOL, clamped) == 0
            {
                crate::error!("Failed to set volume, error: {}", BASS_ErrorGetCode());
            }
        }

        #[cfg(target_os = "android")]
        if let Some(bass) = bass_android::get_bass() {
            unsafe {
                if bass_mixer != 0
                    && (bass.bass_channel_set_attribute)(bass_mixer, BASS_ATTRIB_VOL, clamped) == 0
                {
                    crate::error!(
                        "Failed to set volume, error: {}",
                        (bass.bass_error_get_code)()
                    );
                }
            }
        }
    }

    pub fn equalizer(&self, values: Vec<f32>) {
        crate::info!(
            "Equalizer called with {} bands (not yet implemented)",
            values.len()
        );
        // TODO: Implement BASS_FX equalizer with proper DSP chain
    }

    pub fn reset_equalizer(&self) {
        crate::info!("Reset equalizer (not yet implemented)");
        // TODO: Clear BASS_FX equalizer DSP chain
    }

    pub fn toggle_bit_perfect(&self, enable: bool) {
        crate::info!(
            "Bit-perfect mode toggle (not yet implemented for BASS): {}",
            enable
        );
    }

    pub fn request_sync(&self) {
        self.emit_sync(false);
    }

    pub fn emit_sync(&self, is_reset: bool) {
        app_handle()
            .emit(
                crate::commands::route::MUSIC_PLAYER_SYNC,
                self.get_sync_info(is_reset),
            )
            .unwrap();
    }

    fn play_pause(&self, play: bool) {
        Self::play_pause_inner(&self.bass_mixer, &self.current_stream, play);
        let pos = self.get_current_duration() as u64;
        crate::music::media_session::MediaSession::set_state(play, pos);
    }

    fn play_pause_inner(bass_mixer: &Arc<AtomicU32>, _current_stream: &Arc<AtomicU32>, play: bool) {
        let bm = bass_mixer.load(Ordering::SeqCst);

        #[cfg(desktop)]
        unsafe {
            if bm == 0 {
                return;
            }
            if play {
                if BASS_ChannelPlay(bm, 0) == 0 {
                    crate::error!("Failed to play, error: {}", BASS_ErrorGetCode());
                }
            } else if BASS_ChannelPause(bm) == 0 {
                crate::error!("Failed to pause, error: {}", BASS_ErrorGetCode());
            }
        }

        #[cfg(target_os = "android")]
        if let Some(bass) = bass_android::get_bass() {
            unsafe {
                if bm == 0 {
                    return;
                }
                if play {
                    if (bass.bass_channel_play)(bm, 0) == 0 {
                        crate::error!("Failed to play, error: {}", (bass.bass_error_get_code)());
                    } else {
                        let cs = _current_stream.load(Ordering::SeqCst);
                        let pos = {
                            if cs == 0 {
                                0
                            } else {
                                let bp = (bass.bass_channel_get_position)(cs, BASS_POS_BYTE);
                                ((bass.bass_channel_bytes2seconds)(cs, bp) * 1000.0) as u64
                            }
                        };
                        let _ = app_handle().fluyer().set_media_control_state(true, pos);
                    }
                } else if (bass.bass_channel_pause)(bm) == 0 {
                    crate::error!("Failed to pause, error: {}", (bass.bass_error_get_code)());
                } else {
                    let cs = _current_stream.load(Ordering::SeqCst);
                    let pos = {
                        if cs == 0 {
                            0
                        } else {
                            let bp = (bass.bass_channel_get_position)(cs, BASS_POS_BYTE);
                            ((bass.bass_channel_bytes2seconds)(cs, bp) * 1000.0) as u64
                        }
                    };
                    let _ = app_handle().fluyer().set_media_control_state(false, pos);
                }
            }
        }
    }

    fn clear_track(&self) {
        let bm = self.bass_mixer.load(Ordering::SeqCst);

        #[cfg(desktop)]
        unsafe {
            if bm != 0 {
                BASS_ChannelStop(bm);
                BASS_ChannelSetPosition(bm, 0, BASS_POS_BYTE);
            }
        }
        #[cfg(target_os = "android")]
        if let Some(bass) = bass_android::get_bass() {
            unsafe {
                if bm != 0 {
                    (bass.bass_channel_stop)(bm);
                    (bass.bass_channel_set_position)(bm, 0, BASS_POS_BYTE);
                }
            }
        }

        self.stop_current_stream();
        if let Ok(mut state) = self.state.lock() {
            state.track.clear();
            state.original_track = None;
            state.current_index = None;
        }

        #[cfg(desktop)]
        crate::music::discord_rpc::DiscordRpc::clear();
    }

    fn stop_current_stream(&self) {
        Self::stop_stream(&self.bass_mixer, &self.current_stream, &self.temp_wav_path);
    }

    /// Free current BASS stream and clean up temporary WAV file.
    fn stop_stream(
        bass_mixer: &Arc<AtomicU32>,
        current_stream: &Arc<AtomicU32>,
        temp_wav_path: &Arc<Mutex<Option<PathBuf>>>,
    ) {
        Self::cleanup_temp_wav_inner(temp_wav_path);

        let bm = bass_mixer.load(Ordering::SeqCst);
        let cs = current_stream.load(Ordering::SeqCst);

        #[cfg(desktop)]
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

        #[cfg(target_os = "android")]
        if let Some(bass) = bass_android::get_bass() {
            unsafe {
                if cs != 0 {
                    (bass.bass_channel_stop)(cs);
                    (bass.bass_mixer_channel_remove)(cs);
                    (bass.bass_stream_free)(cs);
                    current_stream.store(0, Ordering::SeqCst);
                }
                if bm != 0 {
                    (bass.bass_channel_set_position)(bm, 0, BASS_POS_BYTE);
                }
            }
        }
    }

    fn cleanup_temp_wav_inner(temp_wav_path: &Arc<Mutex<Option<PathBuf>>>) {
        if let Ok(mut guard) = temp_wav_path.lock() {
            if let Some(path) = guard.take() {
                if path.exists() {
                    match std::fs::remove_file(&path) {
                        Ok(_) => crate::info!("Cleaned up temp WAV file: {}", path.display()),
                        Err(e) => crate::warn!("Failed to remove temp WAV file: {}", e),
                    }
                }
            }
        }
    }

    fn emit_sync_inner(
        bass_mixer: &Arc<AtomicU32>,
        current_stream: &Arc<AtomicU32>,
        state: &Arc<Mutex<PlayerState>>,
        is_reset: bool,
    ) {
        let bm = bass_mixer.load(Ordering::SeqCst);
        let cs = current_stream.load(Ordering::SeqCst);

        let current_position = if is_reset || cs == 0 {
            Some(0.0)
        } else {
            #[cfg(desktop)]
            unsafe {
                let byte_pos = BASS_ChannelGetPosition(cs, BASS_POS_BYTE);
                Some(BASS_ChannelBytes2Seconds(cs, byte_pos) * 1000.0)
            }
            #[cfg(target_os = "android")]
            {
                bass_android::get_bass()
                    .map(|bass| unsafe {
                        let bp = (bass.bass_channel_get_position)(cs, BASS_POS_BYTE);
                        (bass.bass_channel_bytes2seconds)(cs, bp) * 1000.0
                    })
                    .or(Some(0.0))
            }
        };

        let is_playing = if is_reset {
            true
        } else if bm == 0 {
            false
        } else {
            #[cfg(desktop)]
            unsafe {
                BASS_ChannelIsActive(bm) == BASS_ACTIVE_PLAYING
            }
            #[cfg(target_os = "android")]
            {
                bass_android::get_bass()
                    .map(|bass| unsafe { (bass.bass_channel_is_active)(bm) == BASS_ACTIVE_PLAYING })
                    .unwrap_or(false)
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

        app_handle()
            .emit(
                crate::commands::route::MUSIC_PLAYER_SYNC,
                MusicPlayerSync {
                    index,
                    current_position,
                    is_playing,
                    repeat_mode,
                    is_shuffled,
                },
            )
            .unwrap();

        #[cfg(desktop)]
        {
            if index >= 0 {
                if let Ok(state_guard) = state.lock() {
                    if let Some(track) = state_guard.track.get(index as usize) {
                        crate::music::discord_rpc::DiscordRpc::update(
                            crate::music::discord_rpc::ActivityData {
                                title: track
                                    .metadata
                                    .title
                                    .clone()
                                    .unwrap_or_else(|| MusicMetadata::default_title().to_string()),
                                artist: track.metadata.artist.clone(),
                                position_ms: current_position,
                                duration_ms: track.metadata.duration,
                                is_playing,
                            },
                        );
                    }
                }
            } else {
                crate::music::discord_rpc::DiscordRpc::clear();
            }
        }
    }

    /// Load a music file into BASS and add it to the mixer.
    fn setup_sync(
        stream: u32,
        bass_mixer: &Arc<AtomicU32>,
        current_stream: &Arc<AtomicU32>,
        state: &Arc<Mutex<PlayerState>>,
        temp_wav_path: &Arc<Mutex<Option<PathBuf>>>,
    ) {
        if stream == 0 {
            return;
        }

        let sync_data = Box::into_raw(Box::new(SyncData {
            bass_mixer: Arc::clone(bass_mixer),
            current_stream: Arc::clone(current_stream),
            state: Arc::clone(state),
            temp_wav_path: Arc::clone(temp_wav_path),
        }));

        #[cfg(desktop)]
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

        #[cfg(target_os = "android")]
        if let Some(bass) = bass_android::get_bass() {
            unsafe {
                (bass.bass_channel_set_sync)(
                    stream,
                    BASS_SYNC_END | BASS_SYNC_MIXTIME | 0x80000000,
                    0,
                    Some(end_sync_callback),
                    sync_data as *mut _,
                );
                (bass.bass_channel_set_sync)(
                    stream,
                    BASS_SYNC_FREE | 0x80000000,
                    0,
                    Some(free_sync_callback),
                    sync_data as *mut _,
                );
            }
        }
    }

    fn load_music_inner(
        bass_mixer: &Arc<AtomicU32>,
        current_stream: &Arc<AtomicU32>,
        state: &Arc<Mutex<PlayerState>>,
        temp_wav_path: &Arc<Mutex<Option<PathBuf>>>,
        music: MusicMetadata,
        index: usize,
        total_count: usize,
    ) -> bool {
        let bm = bass_mixer.load(Ordering::SeqCst);

        #[cfg(desktop)]
        unsafe {
            let path = CString::new(music.path.clone()).unwrap();
            let stream =
                BASS_StreamCreateFile(false, path.as_ptr() as *const _, 0, 0, BASS_STREAM_DECODE);

            if stream == 0 {
                let bass_error = BASS_ErrorGetCode();
                crate::warn!(
                    "BASS failed to load: {}, error: {}. Trying FFmpeg fallback...",
                    music.path,
                    bass_error
                );

                if let Some(wav_path) = Self::convert_to_pcm_wav(&music.path) {
                    let wav_cstring = CString::new(wav_path.to_string_lossy().as_ref()).unwrap();
                    let wav_stream = BASS_StreamCreateFile(
                        false,
                        wav_cstring.as_ptr() as *const _,
                        0,
                        0,
                        BASS_STREAM_DECODE,
                    );

                    if wav_stream != 0 {
                        let ok = BASS_Mixer_StreamAddChannel(bm, wav_stream, BASS_MIXER_NORAMPIN);
                        if ok != 0 {
                            current_stream.store(wav_stream, Ordering::SeqCst);
                            Self::setup_sync(
                                wav_stream,
                                bass_mixer,
                                current_stream,
                                state,
                                temp_wav_path,
                            );
                            if let Ok(mut guard) = temp_wav_path.lock() {
                                *guard = Some(wav_path.clone());
                            }
                            crate::info!("Successfully loaded via FFmpeg: {}", music.path);
                            return true;
                        } else {
                            crate::error!(
                                "Failed to add FFmpeg-converted channel to mixer: {}, error: {}",
                                music.path,
                                BASS_ErrorGetCode()
                            );
                            BASS_StreamFree(wav_stream);
                        }
                    } else {
                        crate::error!(
                            "BASS failed to load FFmpeg-converted WAV: {}, error: {}",
                            wav_path.display(),
                            BASS_ErrorGetCode()
                        );
                    }
                    let _ = std::fs::remove_file(&wav_path);
                }

                crate::error!(
                    "Failed to load music (both BASS and FFmpeg failed): {}",
                    music.path
                );
                return false;
            }

            let ok = BASS_Mixer_StreamAddChannel(bm, stream, BASS_MIXER_NORAMPIN);
            if ok == 0 {
                crate::error!(
                    "Failed to add channel to mixer: {}, error: {}",
                    music.path,
                    BASS_ErrorGetCode()
                );
                BASS_StreamFree(stream);
                return false;
            }

            current_stream.store(stream, Ordering::SeqCst);
            Self::setup_sync(stream, bass_mixer, current_stream, state, temp_wav_path);
            crate::info!("Successfully loaded: {}", music.path);

            let (is_first, is_last) = if let Ok(s) = state.lock() {
                match s.repeat_mode {
                    RepeatMode::All | RepeatMode::One => (false, false),
                    _ => (index == 0, index == total_count - 1),
                }
            } else {
                (index == 0, index == total_count - 1)
            };

            #[cfg(target_os = "android")]
            {
                let music_clone = music.clone();
                tauri::async_runtime::spawn(async move {
                    crate::music::media_session::MediaSession::update_metadata(
                        &music_clone,
                        true,
                        is_first,
                        is_last,
                    );
                });
            }

            #[cfg(desktop)]
            {
                let music_clone = music.clone();
                tauri::async_runtime::spawn(async move {
                    crate::music::media_session::MediaSession::update_metadata(
                        &music_clone,
                        true,
                        is_first,
                        is_last,
                    )
                    .await;
                });
            }

            return true;
        }

        #[cfg(target_os = "android")]
        {
            if let Some(bass) = bass_android::get_bass() {
                unsafe {
                    let path = CString::new(music.path.clone()).unwrap();
                    let stream = (bass.bass_stream_create_file)(
                        false,
                        path.as_ptr() as *const _,
                        0,
                        0,
                        BASS_STREAM_DECODE,
                    );

                    if stream == 0 {
                        let bass_error = (bass.bass_error_get_code)();
                        crate::warn!(
                            "BASS failed to load: {}, error: {}. Trying FFmpeg fallback...",
                            music.path,
                            bass_error
                        );

                        if let Some(wav_path) = Self::convert_to_pcm_wav_android(&music.path) {
                            let wav_cstring = CString::new(wav_path.as_str()).unwrap();
                            let wav_stream = (bass.bass_stream_create_file)(
                                false,
                                wav_cstring.as_ptr() as *const _,
                                0,
                                0,
                                BASS_STREAM_DECODE,
                            );

                            if wav_stream != 0 {
                                let ok = (bass.bass_mixer_stream_add_channel)(
                                    bm,
                                    wav_stream,
                                    BASS_MIXER_NORAMPIN,
                                );
                                if ok != 0 {
                                    current_stream.store(wav_stream, Ordering::SeqCst);
                                    if let Ok(mut guard) = temp_wav_path.lock() {
                                        *guard = Some(PathBuf::from(&wav_path));
                                    }
                                    crate::info!("Successfully loaded via FFmpeg: {}", music.path);
                                    return true;
                                } else {
                                    crate::error!(
                                        "Failed to add FFmpeg-converted channel to mixer: {}, error: {}",
                                        music.path,
                                        (bass.bass_error_get_code)()
                                    );
                                    (bass.bass_stream_free)(wav_stream);
                                }
                            } else {
                                crate::error!(
                                    "BASS failed to load FFmpeg-converted WAV: {}, error: {}",
                                    wav_path,
                                    (bass.bass_error_get_code)()
                                );
                            }
                            let _ = std::fs::remove_file(&wav_path);
                        }

                        crate::error!(
                            "Failed to load music (both BASS and FFmpeg failed): {}",
                            music.path
                        );
                        return false;
                    }

                    let ok = (bass.bass_mixer_stream_add_channel)(bm, stream, BASS_MIXER_NORAMPIN);
                    if ok == 0 {
                        crate::error!(
                            "Failed to add channel to mixer: {}, error: {}",
                            music.path,
                            (bass.bass_error_get_code)()
                        );
                        (bass.bass_stream_free)(stream);
                        return false;
                    }

                    current_stream.store(stream, Ordering::SeqCst);
                    Self::setup_sync(stream, bass_mixer, current_stream, state, temp_wav_path);
                    crate::info!("Successfully loaded: {}", music.path);

                    let (is_first, is_last) = if let Ok(s) = state.lock() {
                        match s.repeat_mode {
                            RepeatMode::All | RepeatMode::One => (false, false),
                            _ => (index == 0, index == total_count - 1),
                        }
                    } else {
                        (index == 0, index == total_count - 1)
                    };

                    let music_clone = music.clone();
                    tauri::async_runtime::spawn(async move {
                        crate::music::media_session::MediaSession::update_metadata(
                            &music_clone,
                            true,
                            is_first,
                            is_last,
                        )
                        .await;
                    });

                    return true;
                }
            }
            false
        }
    }

    /// Convert audio file to PCM WAV using FFmpegKit on Android
    #[cfg(target_os = "android")]
    fn convert_to_pcm_wav_android(source_path: &str) -> Option<String> {
        crate::info!("Converting {} to PCM WAV via FFmpegKit...", source_path);
        match app_handle()
            .fluyer()
            .audio_convert_to_wav(source_path.to_string())
        {
            Ok(response) => {
                if let Some(path) = response.path {
                    crate::info!("Successfully converted to PCM WAV: {}", path);
                    Some(path)
                } else {
                    crate::error!("FFmpegKit conversion returned no path");
                    None
                }
            }
            Err(e) => {
                crate::error!("FFmpegKit conversion failed: {}", e);
                None
            }
        }
    }

    /// Convert audio file to PCM WAV using FFmpeg for BASS compatibility
    #[cfg(desktop)]
    fn convert_to_pcm_wav(source_path: &str) -> Option<PathBuf> {
        use std::process::Command;

        let ffmpeg_path = {
            #[cfg(target_os = "linux")]
            {
                crate::music::metadata::MusicMetadata::ffmpeg_path().to_path_buf()
            }
            #[cfg(not(target_os = "linux"))]
            {
                app_handle()
                    .path()
                    .resource_dir()
                    .ok()?
                    .join("libs/ffmpeg/bin/ffmpeg")
            }
        };

        let app_data_dir = app_handle().path().app_data_dir().ok()?;
        let temp_dir = app_data_dir.join("temp");
        std::fs::create_dir_all(&temp_dir).ok()?;

        let source_file_name = std::path::Path::new(source_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("audio");
        let output_path = temp_dir.join(format!("{}_converted.wav", source_file_name));
        let _ = std::fs::remove_file(&output_path);

        crate::info!("Converting {} to PCM WAV...", source_path);

        let status = Command::new(&ffmpeg_path)
            .args(&[
                "-y",
                "-i",
                source_path,
                "-vn",
                "-acodec",
                "pcm_s16le",
                "-ar",
                "44100",
                "-ac",
                "2",
                "-f",
                "wav",
            ])
            .arg(&output_path)
            .output();

        match status {
            Ok(output) if output.status.success() => {
                crate::info!(
                    "Successfully converted to PCM WAV: {}",
                    output_path.display()
                );
                Some(output_path)
            }
            Ok(output) => {
                crate::error!(
                    "FFmpeg conversion failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                None
            }
            Err(e) => {
                crate::error!("Failed to run FFmpeg: {}", e);
                None
            }
        }
    }

    /// Update Android media control with current boundary state (is_first, is_last)
    #[cfg(target_os = "android")]
    fn update_android_media_boundaries(&self, current_index: Option<usize>, total_count: usize) {
        if let Some(index) = current_index {
            if total_count > 0 {
                if let Ok(state) = self.state.lock() {
                    if index < state.track.len() {
                        let music = state.track[index].metadata.clone();
                        let is_playing = self.current_stream.load(Ordering::SeqCst) != 0;

                        let (is_first, is_last) = match state.repeat_mode {
                            RepeatMode::All | RepeatMode::One => (false, false),
                            _ => (index == 0, index == total_count - 1),
                        };
                        drop(state);

                        tauri::async_runtime::spawn(async move {
                            crate::music::media_session::MediaSession::update_metadata(
                                &music, is_playing, is_first, is_last,
                            )
                            .await;
                        });
                    }
                }
            }
        }
    }

    fn start_focus_listener(&self) {
        use tauri::Listener;
        let bass_mixer = Arc::clone(&self.bass_mixer);
        let current_stream = Arc::clone(&self.current_stream);
        let state_arc = Arc::clone(&self.state);

        main_window().listen("tauri://focus", move |_| {
            Self::emit_sync_inner(&bass_mixer, &current_stream, &state_arc, false);
        });
    }
}

impl Drop for MusicPlayer {
    fn drop(&mut self) {
        #[cfg(desktop)]
        crate::music::discord_rpc::DiscordRpc::shutdown();

        let bm = self.bass_mixer.load(Ordering::SeqCst);

        #[cfg(desktop)]
        unsafe {
            Self::stop_stream(&self.bass_mixer, &self.current_stream, &self.temp_wav_path);
            if bm != 0 {
                BASS_StreamFree(bm);
                self.bass_mixer.store(0, Ordering::SeqCst);
            }
            BASS_Free();
            crate::info!("BASS cleaned up");
        }

        #[cfg(target_os = "android")]
        if let Some(bass) = bass_android::get_bass() {
            unsafe {
                Self::stop_stream(&self.bass_mixer, &self.current_stream, &self.temp_wav_path);
                if bm != 0 {
                    (bass.bass_stream_free)(bm);
                    self.bass_mixer.store(0, Ordering::SeqCst);
                }
                (bass.bass_free)();
                crate::info!("BASS cleaned up");
            }
        }
    }
}
