use crate::view_models::SettingsViewModel;

impl super::FluyerEngine {
    /// Persisted settings, normalized (`services/settings.rs`).
    pub fn settings_get(&self) -> SettingsViewModel {
        self.settings.load()
    }

    /// Normalizes, writes and returns the canonical values.
    pub fn settings_set(&self, settings: &SettingsViewModel) -> SettingsViewModel {
        self.settings.save(settings)
    }
}
