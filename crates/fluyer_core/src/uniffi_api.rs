use crate::audio::RepeatMode;
use crate::events::EventSink;
use crate::metadata::MusicMetadata;
use crate::view_models::{
    self, AlbumCardViewModel, AlbumDetailViewModel, LyricLine, NativeRepeatMode, PlayViewModel,
    PlayerBarViewModel, ScanStatusViewModel, TrackItemViewModel,
};
use crate::FluyerEngine;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, uniffi::Error)]
pub enum FluyerError {
    InitFailed { message: String },
    OperationFailed { message: String },
}

impl std::fmt::Display for FluyerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FluyerError::InitFailed { message } => write!(f, "Initialization failed: {}", message),
            FluyerError::OperationFailed { message } => write!(f, "Operation failed: {}", message),
        }
    }
}

impl std::error::Error for FluyerError {}

#[derive(uniffi::Enum, Clone, Debug)]
pub enum FluyerEvent {
    PlayerBarUpdated {
        vm: PlayerBarViewModel,
    },
    PlayViewUpdated {
        vm: PlayViewModel,
    },
    TrackChanged {
        track: Option<TrackItemViewModel>,
        index: u64,
    },
    LibraryUpdated,
    ScanProgress {
        vm: ScanStatusViewModel,
    },
    Toast {
        message: String,
    },
    TrackCoverLoaded {
        index: u64,
    },
    AlbumCoverLoaded {
        index: u64,
    },
    LyricsLoaded {
        lyrics: String,
    },
}

#[uniffi::export(callback_interface)]
pub trait FluyerEventListener: Send + Sync + 'static {
    fn on_event(&self, event: FluyerEvent);
}

struct EventSinkBridge {
    listener: Arc<dyn FluyerEventListener>,
}

impl EventSink for EventSinkBridge {
    fn on_player_sync(&self, state: crate::audio::MusicPlayerSync) {
        let duration_ms = state.duration_ms();
        let position_ms = state.position_ms();
        let progress_pct = if duration_ms > 0 {
            (position_ms as f32 / duration_ms as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let time_label = format!(
            "{} / {}",
            view_models::format_time(position_ms),
            view_models::format_time(duration_ms)
        );

        let vm = PlayerBarViewModel {
            track_index: state.index,
            title: String::new(),
            artist: String::new(),
            album: String::new(),
            position_ms,
            duration_ms,
            progress_pct,
            time_label,
            is_playing: state.is_playing,
            repeat_mode: state.repeat_mode.into(),
            is_shuffled: state.is_shuffled,
            volume: 1.0,
        };
        self.listener.on_event(FluyerEvent::PlayerBarUpdated { vm });
    }

    fn on_track_changed(&self, track: Option<MusicMetadata>, index: usize) {
        let view = track.map(|t| TrackItemViewModel::from_metadata(index, &t, true));
        self.listener.on_event(FluyerEvent::TrackChanged {
            track: view,
            index: index as u64,
        });
    }

    fn on_scan_progress(&self, current: usize, total: usize) {
        let vm = ScanStatusViewModel::scanning(current, total);
        self.listener.on_event(FluyerEvent::ScanProgress { vm });
    }

    fn on_toast(&self, message: &str) {
        self.listener.on_event(FluyerEvent::Toast {
            message: message.to_string(),
        });
        if message == "Library scan completed" {
            self.listener.on_event(FluyerEvent::LibraryUpdated);
        }
    }

    fn on_track_cover_loaded(&self, index: usize) {
        self.listener.on_event(FluyerEvent::TrackCoverLoaded {
            index: index as u64,
        });
    }

    fn on_album_cover_loaded(&self, index: usize) {
        self.listener.on_event(FluyerEvent::AlbumCoverLoaded {
            index: index as u64,
        });
    }

    fn on_lyrics_loaded(&self, lyrics: &str) {
        self.listener.on_event(FluyerEvent::LyricsLoaded {
            lyrics: lyrics.to_string(),
        });
    }
}

#[derive(uniffi::Object)]
pub struct FluyerAppEngine {
    inner: Arc<FluyerEngine>,
}

#[uniffi::export]
impl FluyerAppEngine {
    #[uniffi::constructor]
    pub fn new(
        data_dir: String,
        cache_dir: String,
        listener: Option<Box<dyn FluyerEventListener>>,
    ) -> Result<Arc<Self>, FluyerError> {
        let sink: Option<Arc<dyn EventSink>> = listener.map(|l| {
            Arc::new(EventSinkBridge {
                listener: Arc::from(l),
            }) as Arc<dyn EventSink>
        });

        let engine = FluyerEngine::new(Path::new(&data_dir), Path::new(&cache_dir), sink)
            .map_err(|e| FluyerError::InitFailed { message: e })?;

        Ok(Arc::new(Self {
            inner: Arc::new(engine),
        }))
    }

    pub fn play(&self) {
        self.inner.play();
    }

    pub fn pause(&self) {
        self.inner.pause();
    }

    pub fn toggle_play(&self) {
        self.inner.toggle_play();
    }

    pub fn next(&self) {
        self.inner.next();
    }

    pub fn previous(&self) {
        self.inner.previous();
    }

    pub fn seek(&self, position_ms: u64) {
        self.inner.seek(position_ms);
    }

    pub fn get_position(&self) -> u64 {
        self.inner.player.get_sync_info(false).position_ms()
    }

    pub fn set_volume(&self, volume: f32) {
        self.inner.set_volume(volume);
    }

    pub fn get_volume(&self) -> f32 {
        self.inner.get_volume()
    }

    pub fn set_repeat_mode(&self, mode: NativeRepeatMode) {
        self.inner.set_repeat_mode(mode.into());
    }

    pub fn cycle_repeat(&self) {
        let current = self.inner.player.get_sync_info(false).repeat_mode;
        let next = match current {
            RepeatMode::None => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::None,
        };
        self.inner.set_repeat_mode(next);
    }

    pub fn shuffle(&self) {
        self.inner.shuffle();
    }

    pub fn request_sync(&self) {
        self.inner.player.emit_sync(false);
    }

    pub fn scan_directories(&self, directories: Vec<String>) {
        self.inner.scan_and_update(&directories);
    }

    /// Drops every row under `directory` and rebuilds the library synchronously.
    pub fn remove_folder(&self, directory: String) {
        self.inner.remove_folder(&directory);
    }

    pub fn set_discord_enabled(&self, enabled: bool) {
        crate::services::DiscordRpc::set_enabled(enabled);
    }

    /// Queue in play order; `index` on each row is the queue position.
    pub fn get_queue_view(&self) -> Vec<TrackItemViewModel> {
        self.inner.get_queue_view()
    }

    pub fn queue_goto(&self, index: u64) {
        self.inner.queue_goto(index as usize);
    }

    pub fn queue_remove(&self, index: u64) {
        self.inner.queue_remove(index as usize);
    }

    pub fn queue_move(&self, from: u64, to: u64) {
        self.inner.queue_move(from as usize, to as usize);
    }

    pub fn queue_clear(&self) {
        self.inner.queue_clear();
    }

    pub fn get_track_count(&self) -> u64 {
        self.inner.library.read().unwrap().count() as u64
    }

    pub fn get_album_count(&self) -> u64 {
        self.inner.library.read().unwrap().album_count() as u64
    }

    pub fn get_track_view(&self, index: u64) -> Option<TrackItemViewModel> {
        self.inner.get_track_view(index as usize)
    }

    pub fn get_album_card(&self, index: u64) -> Option<AlbumCardViewModel> {
        self.inner.get_album_card(index as usize)
    }

    pub fn get_album_detail(&self, index: u64) -> Option<AlbumDetailViewModel> {
        self.inner.get_album_detail(index as usize)
    }

    pub fn get_album_view(&self, index: u64) -> Option<AlbumCardViewModel> {
        self.inner.get_album_card(index as usize)
    }

    pub fn get_album_tracks(&self, album_index: u64) -> Vec<TrackItemViewModel> {
        self.inner
            .get_album_detail(album_index as usize)
            .map(|detail| detail.tracks)
            .unwrap_or_default()
    }

    pub fn get_player_bar_view(&self) -> PlayerBarViewModel {
        self.inner.get_player_bar_view()
    }

    pub fn get_play_view(&self) -> PlayViewModel {
        self.inner.get_play_view()
    }

    pub fn get_scan_status(&self) -> ScanStatusViewModel {
        ScanStatusViewModel::idle()
    }

    pub fn play_single_from_library(&self, index: u64) {
        self.inner.play_single_from_library(index as usize);
    }

    pub fn play_all_from_library(&self, start_index: u64) {
        self.inner.play_all_from_library(start_index as usize);
    }

    pub fn play_library_tracks(&self, indices: Vec<u64>, start_index: u64) {
        let indices: Vec<usize> = indices.into_iter().map(|i| i as usize).collect();
        self.inner.play_library_tracks(&indices, start_index as usize);
    }

    pub fn play_album(&self, index: u64) {
        self.inner.play_album(index as usize);
    }

    pub fn play_album_track(&self, album_index: u64, track_index: u64) {
        self.inner
            .play_album_track(album_index as usize, track_index as usize);
    }

    pub fn queue_album(&self, index: u64) {
        self.inner.queue_album(index as usize);
    }

    pub fn shuffle_album(&self, index: u64) {
        self.inner.shuffle_album(index as usize);
    }

    pub fn get_lyrics(&self) -> Vec<LyricLine> {
        self.inner.get_parsed_lyrics()
    }

    pub fn get_active_lyric_index(&self, position_ms: u64) -> i32 {
        self.inner.active_lyric_index(position_ms)
    }

    pub fn get_current_image(&self) -> Option<Vec<u8>> {
        self.inner
            .player
            .get_current_track()
            .and_then(|t| self.inner.resolve_track_cover(&t, Some(0)))
    }

    pub fn get_track_image(&self, index: u64) -> Option<Vec<u8>> {
        self.inner
            .library
            .read()
            .unwrap()
            .get_by_index(index as usize)
            .and_then(|t| self.inner.resolve_track_cover(&t, Some(index as usize)))
    }

    pub fn get_album_image(&self, index: u64) -> Option<Vec<u8>> {
        self.inner
            .library
            .read()
            .unwrap()
            .album_get_by_index(index as usize)
            .and_then(|a| self.inner.resolve_album_cover(&a, Some(index as usize)))
    }

    pub fn get_track_thumbnail_rgba(&self, index: u64, width: u32, height: u32) -> Option<Vec<u8>> {
        let bytes = self.get_track_image(index)?;
        view_models::resize_image_rgba(&bytes, width, height).map(|(raw, _, _)| raw)
    }

    pub fn get_album_thumbnail_rgba(&self, index: u64, width: u32, height: u32) -> Option<Vec<u8>> {
        let bytes = self.get_album_image(index)?;
        view_models::resize_image_rgba(&bytes, width, height).map(|(raw, _, _)| raw)
    }

    // ponytail: returns a pre-downscaled JPEG so scroll views never pay a
    // full-resolution decode or a multi-hundred-KB FFI transfer per cell.
    pub fn get_track_thumbnail(&self, index: u64, max_size: u32) -> Option<Vec<u8>> {
        self.inner
            .track_thumbnail(index as usize, max_size)
            .map(|b| b.as_ref().clone())
    }

    pub fn get_album_thumbnail(&self, index: u64, max_size: u32) -> Option<Vec<u8>> {
        self.inner
            .album_thumbnail(index as usize, max_size)
            .map(|b| b.as_ref().clone())
    }

    pub fn get_current_thumbnail(&self, max_size: u32) -> Option<Vec<u8>> {
        self.inner
            .current_thumbnail(max_size)
            .map(|b| b.as_ref().clone())
    }

    // ponytail: async variants offload the blocking decode to the tokio pool so
    // SwiftUI's main thread never stalls while a scroll view populates rows.
    pub async fn load_track_thumbnail(&self, index: u64, max_size: u32) -> Option<Vec<u8>> {
        self.get_track_thumbnail(index, max_size)
    }

    pub async fn load_album_thumbnail(&self, index: u64, max_size: u32) -> Option<Vec<u8>> {
        self.get_album_thumbnail(index, max_size)
    }

    pub async fn load_current_thumbnail(&self, max_size: u32) -> Option<Vec<u8>> {
        self.get_current_thumbnail(max_size)
    }

    pub fn generate_background_for_current(&self, width: u32, height: u32) -> Option<Vec<u8>> {
        let (raw, _, _) = self.inner.generate_background_for_current(width, height);
        Some(raw)
    }

    /// Async twin of `generate_background_for_current`. Blurring and the palette
    /// decode are blocking work, so the UI must not call the sync variant on the
    /// main thread.
    pub async fn load_animated_background(
        &self,
        width: u32,
        height: u32,
    ) -> Option<view_models::AnimatedBackgroundFrame> {
        let (rgba, w, h) = self.inner.generate_background_for_current(width, height);
        Some(view_models::AnimatedBackgroundFrame {
            rgba,
            width: w,
            height: h,
        })
    }
}

#[uniffi::export]
pub fn format_time_label(ms: u64) -> String {
    view_models::format_time(ms)
}

#[uniffi::export]
pub fn parse_lrc_lyrics(lrc_text: String) -> Vec<LyricLine> {
    view_models::parse_lrc(&lrc_text)
}
