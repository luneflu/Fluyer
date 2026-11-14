use fluyer_core::view_models::SettingsViewModel;
use fluyer_core::FluyerEngine;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;
use std::sync::Arc;

/// User settings for the window. The core owns the file (`<data dir>/settings.json`)
/// and every rule that used to live here: JSON shape, folder normalization, dedupe,
/// volume clamp, unknown backdrop value. This type is the observable mirror views bind
/// to; it never touches disk. Fields without a Linux switch yet (music folders, volume,
/// Discord) are carried through unchanged so the shared file stays consistent.
mod imp {
    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::SettingsState)]
    pub struct SettingsState {
        #[property(get, set)]
        pub backdrop_source: RefCell<String>,
        #[property(get, set)]
        pub animated_background: std::cell::Cell<bool>,
        pub engine: RefCell<Option<Arc<FluyerEngine>>>,
        /// Fields the Linux UI does not expose; preserved on every save.
        pub carried: RefCell<SettingsViewModel>,
        pub applying: std::cell::Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SettingsState {
        const NAME: &str = "FluyerSettingsState";
        type Type = super::SettingsState;
    }

    #[glib::derived_properties]
    impl ObjectImpl for SettingsState {
        fn constructed(&self) {
            self.parent_constructed();
            self.backdrop_source.borrow_mut().push_str("artwork");
            self.animated_background.set(true);
        }
    }
}

glib::wrapper! {
    pub struct SettingsState(ObjectSubclass<imp::SettingsState>);
}

impl Default for SettingsState {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsState {
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Reads the stored settings; called once by `AppState` after the engine exists.
    pub fn load(&self, engine: &Arc<FluyerEngine>) {
        let imp = self.imp();
        *imp.engine.borrow_mut() = Some(Arc::clone(engine));
        self.apply(engine.settings_get());
    }

    /// Normalizes + writes through the core and mirrors back the canonical values.
    pub fn save(&self) {
        let imp = self.imp();
        let Some(engine) = imp.engine.borrow().clone() else {
            return;
        };
        if imp.applying.get() {
            return;
        }
        let carried = imp.carried.borrow().clone();
        let stored = engine.settings_set(&SettingsViewModel {
            music_folders: carried.music_folders,
            animated_background: self.animated_background(),
            backdrop_source: self.backdrop_source(),
            discord_rpc: carried.discord_rpc,
            volume: carried.volume,
        });
        self.apply(stored);
    }

    pub fn set_source_and_save(&self, source: &str) {
        self.set_backdrop_source(source);
        self.save();
    }

    fn apply(&self, stored: SettingsViewModel) {
        let imp = self.imp();
        imp.applying.set(true);
        self.set_backdrop_source(&stored.backdrop_source);
        self.set_animated_background(stored.animated_background);
        *imp.carried.borrow_mut() = stored;
        imp.applying.set(false);
    }
}
