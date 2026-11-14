//! The one settings file (`<data_dir>/settings.json`), owned by the core.
//!
//! UIs used to each hand-roll the same JSON reader/writer and drifted apart in
//! field names and defaults. The core now normalizes folders, clamps the volume and
//! maps an unknown backdrop source to `artwork`, so every platform reads back the
//! same values it stored.

use crate::view_models::SettingsViewModel;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const FILE_NAME: &str = "settings.json";

/// On-disk shape. Every field is optional so an older or partial file keeps the
/// defaults; the `alias`es accept the snake_case keys the Linux UI used to write.
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct SettingsFile {
    #[serde(alias = "music_folders")]
    music_folders: Vec<String>,
    #[serde(alias = "animated_background")]
    animated_background: Option<bool>,
    #[serde(alias = "backdrop_source")]
    backdrop_source: Option<String>,
    #[serde(alias = "discord_rpc")]
    discord_rpc: Option<bool>,
    volume: Option<f32>,
}

pub struct SettingsService {
    path: PathBuf,
}

impl SettingsService {
    pub fn new(data_dir: &Path) -> Self {
        Self { path: data_dir.join(FILE_NAME) }
    }

    /// Stored settings, or defaults when the file is missing or unreadable. An
    /// unreadable file is kept as `settings.json.bad` so the next save can't
    /// silently destroy it.
    pub fn load(&self) -> SettingsViewModel {
        let Ok(text) = std::fs::read_to_string(&self.path) else {
            return SettingsViewModel::default();
        };
        match serde_json::from_str::<SettingsFile>(&text) {
            Ok(file) => file.into_view_model(),
            Err(_) => {
                let _ = std::fs::copy(&self.path, self.path.with_extension("json.bad"));
                SettingsViewModel::default()
            }
        }
    }

    /// Normalizes, writes atomically and returns what was stored, so callers can
    /// display the canonical values (normalized folders, clamped volume).
    pub fn save(&self, settings: &SettingsViewModel) -> SettingsViewModel {
        let canonical = canonical(settings);
        let file = SettingsFile {
            music_folders: canonical.music_folders.clone(),
            animated_background: Some(canonical.animated_background),
            backdrop_source: Some(canonical.backdrop_source.clone()),
            discord_rpc: Some(canonical.discord_rpc),
            volume: Some(canonical.volume),
        };
        if let Ok(json) = serde_json::to_string_pretty(&file) {
            let tmp = self.path.with_extension("json.tmp");
            if std::fs::write(&tmp, json).is_ok() && std::fs::rename(&tmp, &self.path).is_err() {
                let _ = std::fs::remove_file(&tmp);
            }
        }
        canonical
    }
}

impl SettingsFile {
    fn into_view_model(self) -> SettingsViewModel {
        let defaults = SettingsViewModel::default();
        SettingsViewModel {
            music_folders: self.music_folders,
            animated_background: self.animated_background.unwrap_or(defaults.animated_background),
            backdrop_source: normalize_backdrop(
                self.backdrop_source.as_deref().unwrap_or(&defaults.backdrop_source),
            ),
            discord_rpc: self.discord_rpc.unwrap_or(defaults.discord_rpc),
            volume: normalize_volume(self.volume.unwrap_or(defaults.volume)),
        }
    }
}

/// Trimmed, without a trailing separator, root kept as-is (`/`, `C:\`), empties
/// dropped, duplicates removed case-insensitively (first spelling wins).
pub fn canonical(settings: &SettingsViewModel) -> SettingsViewModel {
    let mut folders: Vec<String> = Vec::new();
    for raw in &settings.music_folders {
        let path = normalize_folder(raw);
        if path.is_empty() {
            continue;
        }
        if !folders.iter().any(|kept| kept.eq_ignore_ascii_case(&path)) {
            folders.push(path);
        }
    }
    SettingsViewModel {
        music_folders: folders,
        animated_background: settings.animated_background,
        backdrop_source: normalize_backdrop(&settings.backdrop_source),
        discord_rpc: settings.discord_rpc,
        volume: normalize_volume(settings.volume),
    }
}

pub fn normalize_folder(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.len() <= 1 {
        // "" and "/" have nothing to strip.
        return trimmed.to_string();
    }
    let stripped = trimmed.trim_end_matches(['/', '\\']);
    if stripped.len() == 2 && stripped.ends_with(':') {
        // Windows drive root: "C:" -> "C:\" rather than a degenerate root-less path.
        return format!("{}\\", stripped);
    }
    if stripped.is_empty() {
        "/".to_string()
    } else {
        stripped.to_string()
    }
}

/// Unknown values degrade to `artwork` instead of failing the whole file.
pub fn normalize_backdrop(source: &str) -> String {
    if source.trim().eq_ignore_ascii_case("blocks") {
        "blocks".to_string()
    } else {
        "artwork".to_string()
    }
}

pub fn normalize_volume(volume: f32) -> f32 {
    if volume.is_nan() {
        1.0
    } else {
        volume.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "fluyer-settings-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn round_trips_every_field() {
        let dir = temp_dir();
        let service = SettingsService::new(&dir);
        let saved = service.save(&SettingsViewModel {
            music_folders: vec!["/Music/".into(), "/Other".into()],
            animated_background: false,
            backdrop_source: "blocks".into(),
            discord_rpc: false,
            volume: 0.4,
        });
        assert_eq!(saved.music_folders, vec!["/Music", "/Other"]);

        let loaded = service.load();
        assert_eq!(loaded, saved);
        assert_eq!(loaded.volume, 0.4);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn missing_and_partial_files_keep_defaults_and_clamp() {
        let dir = temp_dir();
        let service = SettingsService::new(&dir);
        assert_eq!(service.load(), SettingsViewModel::default());

        std::fs::write(dir.join(FILE_NAME), r#"{"volume": 7}"#).unwrap();
        let loaded = service.load();
        assert_eq!(loaded.volume, 1.0);
        assert!(loaded.discord_rpc);
        assert_eq!(loaded.backdrop_source, "artwork");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn snake_case_file_from_the_linux_ui_still_loads() {
        let dir = temp_dir();
        std::fs::write(
            dir.join(FILE_NAME),
            r#"{"backdrop_source": "blocks", "animated_background": false}"#,
        )
        .unwrap();
        let loaded = SettingsService::new(&dir).load();
        assert_eq!(loaded.backdrop_source, "blocks");
        assert!(!loaded.animated_background);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn unknown_backdrop_and_corrupt_file_degrade_without_losing_the_file() {
        let dir = temp_dir();
        let path = dir.join(FILE_NAME);
        std::fs::write(&path, r#"{"backdropSource": "shader", "discordRpc": false}"#).unwrap();
        let service = SettingsService::new(&dir);
        let loaded = service.load();
        assert_eq!(loaded.backdrop_source, "artwork");
        assert!(!loaded.discord_rpc);

        std::fs::write(&path, "{not json").unwrap();
        assert_eq!(service.load(), SettingsViewModel::default());
        assert_eq!(
            std::fs::read_to_string(dir.join("settings.json.bad")).unwrap(),
            "{not json"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn folders_normalize_and_dedupe() {
        let canon = canonical(&SettingsViewModel {
            music_folders: vec![
                "/Music".into(),
                "/music/".into(),
                "  ".into(),
                "C:\\".into(),
                "/".into(),
            ],
            ..SettingsViewModel::default()
        });
        assert_eq!(canon.music_folders, vec!["/Music", "C:\\", "/"]);
        assert_eq!(normalize_folder("D:\\Songs\\"), "D:\\Songs");
    }
}
