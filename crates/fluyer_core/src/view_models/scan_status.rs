use serde::{Deserialize, Serialize};

#[derive(uniffi::Record, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScanStatusViewModel {
    pub is_scanning: bool,
    pub current: u64,
    pub total: u64,
    pub progress_pct: f32,
    pub status_label: String,
}

impl ScanStatusViewModel {
    pub fn idle() -> Self {
        Self {
            is_scanning: false,
            current: 0,
            total: 0,
            progress_pct: 0.0,
            status_label: "Ready".to_string(),
        }
    }

    pub fn scanning(current: usize, total: usize) -> Self {
        let progress_pct = if total > 0 {
            (current as f32 / total as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let status_label = if total > 0 {
            format!("Scanning: {} / {} files", current, total)
        } else {
            "Scanning library...".to_string()
        };

        Self {
            is_scanning: true,
            current: current as u64,
            total: total as u64,
            progress_pct,
            status_label,
        }
    }

    pub fn completed(total: usize) -> Self {
        Self {
            is_scanning: false,
            current: total as u64,
            total: total as u64,
            progress_pct: 1.0,
            status_label: format!("Scanned {} files", total),
        }
    }
}
