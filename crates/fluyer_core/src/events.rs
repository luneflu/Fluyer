use crate::audio::MusicPlayerSync;
use crate::metadata::MusicMetadata;

pub trait EventSink: Send + Sync + 'static {
    fn on_player_sync(&self, state: MusicPlayerSync);
    fn on_track_changed(&self, track: Option<MusicMetadata>, index: usize);
    fn on_scan_progress(&self, current: usize, total: usize);
    fn on_toast(&self, message: &str);
}
