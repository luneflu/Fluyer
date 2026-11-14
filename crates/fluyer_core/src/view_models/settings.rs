use serde::{Deserialize, Serialize};

/// Persisted user settings, owned by the core so every UI reads and writes the same
/// file with the same normalization (`services/settings.rs`).
///
/// The defaults are the ones every platform shipped with: no folders, animated
/// backdrop on, artwork as the backdrop input, Discord presence on, full volume.
#[derive(uniffi::Record, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsViewModel {
    pub music_folders: Vec<String>,
    pub animated_background: bool,
    /// `"artwork"` or `"blocks"`.
    pub backdrop_source: String,
    pub discord_rpc: bool,
    pub volume: f32,
}

impl Default for SettingsViewModel {
    fn default() -> Self {
        Self {
            music_folders: Vec::new(),
            animated_background: true,
            backdrop_source: "artwork".to_string(),
            discord_rpc: true,
            volume: 1.0,
        }
    }
}
