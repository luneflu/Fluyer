use super::FluyerAppEngine;
use crate::view_models::SettingsViewModel;

#[uniffi::export]
impl FluyerAppEngine {
    pub fn settings_get(&self) -> SettingsViewModel {
        self.inner.settings_get()
    }

    /// Persists and returns what was stored (normalized folders, clamped volume).
    pub fn settings_set(&self, settings: SettingsViewModel) -> SettingsViewModel {
        self.inner.settings_set(&settings)
    }
}
