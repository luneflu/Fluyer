use crate::audio::MusicPlayerSync;
use crate::metadata::MusicMetadata;

pub trait EventSink: Send + Sync + 'static {
    fn on_player_sync(&self, state: MusicPlayerSync);
    fn on_track_changed(&self, track: Option<MusicMetadata>, index: usize);
    fn on_scan_progress(&self, current: usize, total: usize);
    fn on_toast(&self, message: &str);
    fn on_track_cover_loaded(&self, _index: usize) {}
    fn on_album_cover_loaded(&self, _index: usize) {}
    fn on_lyrics_loaded(&self, _lyrics: &str) {}
}
