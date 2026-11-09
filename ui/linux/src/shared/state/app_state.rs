use super::{CoverState, LibraryState, PlaybackState, QueueState, SettingsState};
use crate::events::AppEvent;
use fluyer_core::FluyerEngine;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub struct AppState {
    pub engine: Arc<FluyerEngine>,
    pub playback: PlaybackState,
    pub library: LibraryState,
    pub queue: QueueState,
    pub covers: CoverState,
    pub settings: SettingsState,
    pub toast_overlay: RefCell<Option<adw::ToastOverlay>>,
    pub backdrop_redraw_trigger: RefCell<Option<gtk::Widget>>,
}

impl AppState {
    pub fn new(engine: Arc<FluyerEngine>) -> Rc<Self> {
        let app = Rc::new(Self {
            engine,
            playback: PlaybackState::new(),
            library: LibraryState::new(),
            queue: QueueState::new(),
            covers: CoverState::new(),
            settings: SettingsState::new(),
            toast_overlay: RefCell::new(None),
            backdrop_redraw_trigger: RefCell::new(None),
        });

        app.init();
        app
    }

    pub fn init(&self) {
        let bar = self.engine.player_get_bar();
        self.playback.apply_bar(&bar);
        self.playback.sync_lyrics(&self.engine);
        self.queue.reload(&self.engine);
        self.library.reload(&self.engine);
    }

    pub fn handle_event(&self, event: AppEvent) {
        match event {
            AppEvent::PlayerSync(sync) => {
                self.playback.apply_sync(&sync, &self.engine);
            }
            AppEvent::TrackChanged(_meta, _index) => {
                // Drop the cached `current:*` textures first: the bar/play covers pick the
                // new one up from the property re-bind below. Invalidating after
                // `apply_bar` leaves them showing the previous track's cover.
                self.covers.invalidate_current();
                let bar = self.engine.player_get_bar();
                self.playback.apply_bar(&bar);
                self.playback.sync_lyrics(&self.engine);
                self.queue.reload(&self.engine);
                self.playback.set_cover_tick(self.playback.cover_tick().wrapping_add(1));
                if let Some(w) = self.backdrop_redraw_trigger.borrow().as_ref() {
                    w.queue_draw();
                }
            }
            // Progress only: `current` counts changed files, so it may never reach `total`.
            AppEvent::ScanProgress(current, total) => {
                self.library.set_is_scanning(true);
                self.library
                    .set_scan_progress_label(format!("Scanning ({}/{})", current, total));
            }
            AppEvent::Toast(msg) => {
                // ponytail: core has no scan-finished event; this toast is sent right
                // after `library.rebuild` (same trick as uniffi_api/events.rs).
                if msg == "Library scan completed" {
                    self.library.set_is_scanning(false);
                    self.library.reload(&self.engine);
                }
                self.show_toast(&msg);
            }
            AppEvent::TrackCoverLoaded(_) | AppEvent::AlbumCoverLoaded(_) => {
                self.covers.clear();
                if let Some(w) = self.backdrop_redraw_trigger.borrow().as_ref() {
                    w.queue_draw();
                }
            }
            AppEvent::LyricsLoaded(_) => {
                self.playback.sync_lyrics(&self.engine);
            }
        }
    }

    pub fn show_toast(&self, msg: &str) {
        if let Some(overlay) = self.toast_overlay.borrow().as_ref() {
            let toast = adw::Toast::new(msg);
            overlay.add_toast(toast);
        }
    }

    pub fn trigger_backdrop_redraw(&self) {
        if let Some(w) = self.backdrop_redraw_trigger.borrow().as_ref() {
            w.queue_draw();
        }
    }
}
