use crate::events::EventSink;
use crate::metadata::MusicMetadata;
use crate::view_models::{
    self, PlayViewModel, PlayerBarViewModel, ScanStatusViewModel, TrackItemViewModel,
};
use std::sync::Arc;

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

/// Adapts the core's `EventSink` to the single `on_event` callback the UI implements.
pub(super) struct EventSinkBridge {
    listener: Arc<dyn FluyerEventListener>,
}

impl EventSinkBridge {
    pub(super) fn new(listener: Box<dyn FluyerEventListener>) -> Self {
        Self { listener: Arc::from(listener) }
    }
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
