use crate::music::metadata::MusicMetadata;
use crate::state::{app_handle, main_window};
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
}

#[derive(Debug, Clone)]
struct PlayerState {
    track: Vec<TrackItem>,
    current_index: Option<usize>,
    repeat_mode: RepeatMode,
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

    cs_arc.store(0, Ordering::SeqCst);
    crate::info!("Track ended, playing next");

    tauri::async_runtime::spawn_blocking(move || {
        let next_index = {
            let state = match st.lock() {
                Ok(s) => s,
                Err(e) => {
                    crate::error!("Failed to lock player state: {}", e);
                    return;
                }
            };
            match (state.current_index, state.repeat_mode) {
                (Some(current), RepeatMode::One) => Some(current),
                (Some(current), _) if current + 1 < state.track.len() => Some(current + 1),
                (Some(_), RepeatMode::All) => Some(0),
                _ => None,
            }
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

            let cs2 = cs_arc.load(Ordering::SeqCst);
            #[cfg(desktop)]
            unsafe {
                if cs2 != 0 {
                    BASS_Mixer_ChannelRemove(cs2);
                    BASS_StreamFree(cs2);
                    cs_arc.store(0, Ordering::SeqCst);
                }
            }
            #[cfg(target_os = "android")]
            if let Some(bass) = bass_android::get_bass() {
                unsafe {
                    if cs2 != 0 {
                        (bass.bass_mixer_channel_remove)(cs2);
                        (bass.bass_stream_free)(cs2);
                        cs_arc.store(0, Ordering::SeqCst);
                    }
                }
            }

            if MusicPlayer::load_music_inner(&bm, &cs_arc, &st, &twp, music, index, total_count) {
                if let Ok(mut state) = st.lock() {
                    state.current_index = Some(index);
                }
                MusicPlayer::emit_sync_inner(&bm, &cs_arc, &st, true);
            }
        } else {
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

            let mixer = BASS_Mixer_StreamCreate(44100, 2, BASS_SAMPLE_FLOAT);
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

                        let mixer = (bass.bass_mixer_stream_create)(44100, 2, BASS_SAMPLE_FLOAT);
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
        self.play_pause(true);
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
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::SystemTime;

        let seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(42);

        let mut rng = seed as usize;
        if let Ok(mut state) = self.state.lock() {
            let len = state.track.len();
            for i in (1..len).rev() {
                let mut h = DefaultHasher::new();
                (rng ^ i).hash(&mut h);
                rng = h.finish() as usize;
                let j = rng % (i + 1);
                state.track.swap(i, j);
            }
            state.current_index = if len > 0 { Some(0) } else { None };
        }
        self.goto_track(0);
    }

    pub fn set_repeat_mode(&self, mode: RepeatMode) {
        if let Ok(mut state) = self.state.lock() {
            state.repeat_mode = mode;
        }
        self.emit_sync(false);
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

                    let sync_info = self.get_sync_info(false);
                    crate::debug!(
                        "Updating media control state after seek: is_playing={}, position={}ms",
                        sync_info.is_playing,
                        position
                    );
                    crate::music::media_session::MediaSession::set_state(
                        sync_info.is_playing,
                        position,
                    );
                }
            }
        }
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

        let (index, repeat_mode) = self
            .state
            .lock()
            .map(|s| {
                (
                    s.current_index.map(|i| i as i64).unwrap_or(-1),
                    s.repeat_mode,
                )
            })
            .unwrap_or((-1, RepeatMode::None));

        MusicPlayerSync {
            index,
            current_position,
            is_playing,
            repeat_mode,
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
            for music in track {
                state.track.push(TrackItem { metadata: music });
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

        if let Some(current) = state.current_index {
            if current == index {
                drop(state);
                self.stop_current_stream();
                if let Ok(mut state) = self.state.lock() {
                    state.current_index = None;
                    state.track.remove(index);
                }
                return;
            }
        }

        state.track.remove(index);

        if let Some(current) = state.current_index {
            if index < current {
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
                let state = match state_arc.lock() {
                    Ok(s) => s,
                    Err(e) => {
                        crate::error!("Failed to lock player state: {}", e);
                        return;
                    }
                };
                match (state.current_index, state.repeat_mode) {
                    (Some(current), RepeatMode::One) if !from_user => Some(current),
                    (Some(current), _) if current + 1 < state.track.len() => Some(current + 1),
                    (Some(_), RepeatMode::All) => Some(0),
                    (Some(_), _) if from_user => Some(0),
                    _ => None,
                }
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
                    Some(current) if current > 0 => Some(current - 1),
                    Some(_) if !state.track.is_empty() => Some(state.track.len() - 1),
                    _ => None,
                }
            };

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
            state.track.insert(to, item);

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
                        let cs = current_stream.load(Ordering::SeqCst);
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
                    let cs = current_stream.load(Ordering::SeqCst);
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
            state.current_index = None;
        }
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

        let (index, repeat_mode) = state
            .lock()
            .map(|s| {
                (
                    s.current_index.map(|i| i as i64).unwrap_or(-1),
                    s.repeat_mode,
                )
            })
            .unwrap_or((-1, RepeatMode::None));

        app_handle()
            .emit(
                crate::commands::route::MUSIC_PLAYER_SYNC,
                MusicPlayerSync {
                    index,
                    current_position,
                    is_playing,
                    repeat_mode,
                },
            )
            .unwrap();
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
        _index: usize,
        _total_count: usize,
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

            #[cfg(target_os = "android")]
            {
                let music_clone = music.clone();
                tauri::async_runtime::spawn(async move {
                    let handle = app_handle();
                    let image_path =
                        match handle.fluyer().metadata_get_image(music_clone.path.clone()) {
                            Ok(res) => res.path,
                            Err(_) => None,
                        };
                    let _ = handle.fluyer().update_media_control(
                        music_clone.title.unwrap_or("Unknown".to_string()),
                        music_clone.artist.unwrap_or("Unknown".to_string()),
                        music_clone.album.unwrap_or("Unknown".to_string()),
                        music_clone.duration.unwrap_or(0) as u64,
                        image_path,
                        true,
                    );
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

                    let music_clone = music.clone();
                    tauri::async_runtime::spawn(async move {
                        let handle = app_handle();
                        let image_path =
                            match handle.fluyer().metadata_get_image(music_clone.path.clone()) {
                                Ok(res) => res.path,
                                Err(_) => None,
                            };
                        let _ = handle.fluyer().update_media_control(
                            music_clone.title.unwrap_or("Unknown".to_string()),
                            music_clone.artist.unwrap_or("Unknown".to_string()),
                            music_clone.album.unwrap_or("Unknown".to_string()),
                            music_clone.duration.unwrap_or(0) as u64,
                            image_path,
                            true,
                            index == 0,
                            index == total_count - 1,
                        );
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
                PathBuf::from("/usr/lib/fluyer/ffmpeg")
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
                let is_first = index == 0;
                let is_last = index == total_count - 1;

                if let Ok(state) = self.state.lock() {
                    if index < state.track.len() {
                        let music = state.track[index].metadata.clone();
                        let is_playing = self.current_stream.load(Ordering::SeqCst) != 0;
                        drop(state);

                        tauri::async_runtime::spawn(async move {
                            crate::music::media_session::MediaSession::update_metadata(
                                &music, is_playing, is_first, is_last,
                            );
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
