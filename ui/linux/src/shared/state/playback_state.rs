use crate::shared::models::LyricLineObject;
use fluyer_core::audio::player::MusicPlayerSync;
use fluyer_core::view_models::{format_time, NativeRepeatMode, PlayerBarViewModel};
use fluyer_core::FluyerEngine;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::PlaybackState)]
    pub struct PlaybackState {
        #[property(get, set)]
        pub is_playing: Cell<bool>,
        #[property(get, set)]
        pub title: RefCell<String>,
        #[property(get, set)]
        pub artist: RefCell<String>,
        #[property(get, set)]
        pub album: RefCell<String>,
        #[property(get, set)]
        pub position_ms: Cell<u64>,
        #[property(get, set)]
        pub duration_ms: Cell<u64>,
        #[property(get, set)]
        pub progress_pct: Cell<f32>,
        #[property(get, set)]
        pub time_label: RefCell<String>,
        #[property(get, set)]
        pub volume: Cell<f32>,
        #[property(get, set)]
        pub repeat_mode: Cell<u32>, // 0 = None, 1 = All, 2 = One
        #[property(get, set)]
        pub is_shuffled: Cell<bool>,
        #[property(get, set)]
        pub show_play_view: Cell<bool>,
        #[property(get, set)]
        pub current_lyric_index: Cell<i32>,
        /// Bumped whenever the current track changes, so covers reload.
        #[property(get, set)]
        pub cover_tick: Cell<u32>,

        pub lyrics: RefCell<Option<gio::ListStore>>,
        pub last_audible_volume: Cell<f32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaybackState {
        const NAME: &'static str = "FluyerPlaybackState";
        type Type = super::PlaybackState;
    }

    #[glib::derived_properties]
    impl ObjectImpl for PlaybackState {
        fn constructed(&self) {
            self.parent_constructed();
            self.volume.set(1.0);
            self.last_audible_volume.set(1.0);
            *self.title.borrow_mut() = "No track playing".to_string();
            *self.artist.borrow_mut() = "Fluyer".to_string();
            *self.time_label.borrow_mut() = "0:00 / 0:00".to_string();
            *self.lyrics.borrow_mut() = Some(gio::ListStore::new::<LyricLineObject>());
        }
    }
}

glib::wrapper! {
    pub struct PlaybackState(ObjectSubclass<imp::PlaybackState>);
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaybackState {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn lyrics_store(&self) -> gio::ListStore {
        self.imp().lyrics.borrow().as_ref().unwrap().clone()
    }

    pub fn apply_bar(&self, bar: &PlayerBarViewModel) {
        self.set_is_playing(bar.is_playing);
        self.set_title(if bar.title.is_empty() {
            "No track playing"
        } else {
            &bar.title
        });
        self.set_artist(if bar.artist.is_empty() {
            "Fluyer"
        } else {
            &bar.artist
        });
        self.set_album(&*bar.album);
        self.set_position_ms(bar.position_ms);
        self.set_duration_ms(bar.duration_ms);
        self.set_progress_pct(bar.progress_pct);
        self.set_time_label(&*bar.time_label);
        self.set_volume(bar.volume);
        self.set_repeat_mode(match bar.repeat_mode {
            NativeRepeatMode::None => 0,
            NativeRepeatMode::All => 1,
            NativeRepeatMode::One => 2,
        });
        self.set_is_shuffled(bar.is_shuffled);
    }

    pub fn apply_sync(&self, sync: &MusicPlayerSync, engine: &FluyerEngine) {
        let is_playing = sync.is_playing;
        self.set_is_playing(is_playing);

        if let (Some(cur), Some(dur)) = (sync.current_position, sync.duration) {
            let cur_ms = (cur * 1000.0).max(0.0) as u64;
            let dur_ms = (dur * 1000.0).max(0.0) as u64;
            let pct = if dur_ms > 0 {
                (cur_ms as f32 / dur_ms as f32).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let label = format!("{} / {}", format_time(cur_ms), format_time(dur_ms));

            self.set_position_ms(cur_ms);
            self.set_duration_ms(dur_ms);
            self.set_progress_pct(pct);
            self.set_time_label(&*label);

            self.set_active_lyric(engine.lyrics_get_active_index(cur_ms));
        }
    }

    /// Moves the lyric cursor; clamps to the loaded lines (core may answer for
    /// lyrics the store hasn't synced yet).
    pub fn set_active_lyric(&self, idx: i32) {
        let store = self.lyrics_store();
        let idx = if idx >= 0 && (idx as u32) < store.n_items() { idx } else { -1 };
        if self.current_lyric_index() == idx {
            return;
        }
        // Flags first, then the index: listeners read `is_active` when the index notify
        // fires, so emitting first made the highlight lag one line behind the scroll.
        for i in 0..store.n_items() {
            if let Some(item) = store.item(i).and_downcast::<LyricLineObject>() {
                item.set_is_active(i as i32 == idx);
            }
        }
        self.set_current_lyric_index(idx);
    }

    pub fn sync_lyrics(&self, engine: &FluyerEngine) {
        let lines = engine.lyrics_get();
        let store = self.lyrics_store();
        // Reset the cursor before the store changes, set it again after it is filled:
        // an index past the store end makes ListView::scroll_to assert.
        self.set_current_lyric_index(-1);
        store.remove_all();
        for line in &lines {
            store.append(&LyricLineObject::new(line.timestamp_ms, &line.text, false));
        }
        self.set_active_lyric(engine.lyrics_get_active_index(self.position_ms()));
    }

    pub fn toggle_play(&self, engine: &FluyerEngine) {
        engine.player_toggle_play();
    }

    pub fn next(&self, engine: &FluyerEngine) {
        engine.player_next();
    }

    pub fn previous(&self, engine: &FluyerEngine) {
        engine.player_previous();
    }

    pub fn seek_pct(&self, engine: &FluyerEngine, pct: f32) {
        let dur = self.duration_ms();
        if dur > 0 {
            let target_ms = ((dur as f64) * (pct as f64).clamp(0.0, 1.0)) as u64;
            self.seek_ms(engine, target_ms);
        }
    }

    pub fn seek_ms(&self, engine: &FluyerEngine, ms: u64) {
        engine.player_seek(ms);
    }

    pub fn set_volume_val(&self, engine: &FluyerEngine, vol: f32) {
        let clamped = vol.clamp(0.0, 1.0);
        if clamped > 0.001 {
            self.imp().last_audible_volume.set(clamped);
        }
        self.set_volume(clamped);
        engine.player_set_volume(clamped);
    }

    pub fn toggle_mute(&self, engine: &FluyerEngine) {
        if self.volume() <= 0.001 {
            let restore = self.imp().last_audible_volume.get().max(0.05);
            self.set_volume_val(engine, restore);
        } else {
            self.set_volume_val(engine, 0.0);
        }
    }

    pub fn cycle_repeat(&self, engine: &FluyerEngine) {
        engine.player_cycle_repeat();
    }

    pub fn shuffle(&self, engine: &FluyerEngine) {
        engine.player_shuffle();
    }
}
