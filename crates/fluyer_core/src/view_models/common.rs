use serde::{Deserialize, Serialize};

#[derive(uniffi::Enum, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum NativeRepeatMode {
    #[default]
    None = 0,
    All = 1,
    One = 2,
}

impl From<crate::audio::RepeatMode> for NativeRepeatMode {
    fn from(m: crate::audio::RepeatMode) -> Self {
        match m {
            crate::audio::RepeatMode::None => NativeRepeatMode::None,
            crate::audio::RepeatMode::All => NativeRepeatMode::All,
            crate::audio::RepeatMode::One => NativeRepeatMode::One,
        }
    }
}

impl From<NativeRepeatMode> for crate::audio::RepeatMode {
    fn from(m: NativeRepeatMode) -> Self {
        match m {
            NativeRepeatMode::None => crate::audio::RepeatMode::None,
            NativeRepeatMode::All => crate::audio::RepeatMode::All,
            NativeRepeatMode::One => crate::audio::RepeatMode::One,
        }
    }
}

#[derive(uniffi::Record, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LyricLine {
    pub timestamp_ms: u64,
    pub text: String,
}

#[derive(uniffi::Record, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
