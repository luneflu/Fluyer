use std::env::temp_dir;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;

use once_cell::sync::Lazy;
use tauri::Emitter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

impl Level {
    fn as_str(&self) -> &'static str {
        match self {
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Info => "INFO",
            Level::Debug => "DEBUG",
            Level::Trace => "TRACE",
        }
    }
}

fn level_color(level: Level) -> &'static str {
    match level {
        Level::Error => "\x1b[31m", // Red
        Level::Warn => "\x1b[33m",  // Yellow
        Level::Info => "\x1b[32m",  // Green
        Level::Debug => "\x1b[36m", // Cyan
        Level::Trace => "\x1b[37m", // White
    }
}

const RESET: &str = "\x1b[0m";
const MAX_LOGS: usize = 1000;

// Global max level filter (4 = Debug by default)
static MAX_LEVEL: AtomicU8 = AtomicU8::new(4);
static LOG_BUFFER: Lazy<Mutex<Vec<(String, String)>>> =
    Lazy::new(|| Mutex::new(Vec::with_capacity(MAX_LOGS)));

pub fn init() {
    // Set max level to Debug
    MAX_LEVEL.store(4, Ordering::Relaxed);
}

#[doc(hidden)]
pub fn _log(level: Level, target: &str, args: std::fmt::Arguments) {
    // Check if this level is enabled
    if (level as u8) > MAX_LEVEL.load(Ordering::Relaxed) {
        return;
    }

    let color = level_color(level);
    let message = format!("{}", args);
    let full_message = format!("{} - {}", target, message);

    println!("{}[{}]{} {}", color, level.as_str(), RESET, full_message);

    if let Ok(mut buffer) = LOG_BUFFER.lock() {
        if buffer.len() >= MAX_LOGS {
            buffer.remove(0);
        }
        buffer.push((level.as_str().to_string(), full_message.clone()));
    }

    if let Some(handle) = crate::state::try_app_handle() {
        let _ = handle.emit(
            crate::commands::route::LOG,
            [level.as_str(), full_message.as_str()],
        );
    }
}

pub fn get_buffered_logs() -> Vec<(String, String)> {
    LOG_BUFFER.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::logger::_log($crate::logger::Level::Error, module_path!(), format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::logger::_log($crate::logger::Level::Warn, module_path!(), format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::logger::_log($crate::logger::Level::Info, module_path!(), format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::logger::_log($crate::logger::Level::Debug, module_path!(), format_args!($($arg)*))
    };
}

macro_rules! trace {
    ($($arg:tt)*) => {
        $crate::logger::_log($crate::logger::Level::Trace, module_path!(), format_args!($($arg)*))
    };
}

#[allow(dead_code)]
pub fn get_log_name() -> String {
    "fluyer.log".to_string()
}

#[allow(dead_code)]
pub fn get_mpv_log_name() -> String {
    "fluyer-mpv.log".to_string()
}

#[allow(dead_code)]
pub fn get_log_path() -> String {
    temp_dir().join(get_log_name()).display().to_string()
}

#[allow(dead_code)]
pub fn get_mpv_log_path() -> String {
    temp_dir().join(get_mpv_log_name()).display().to_string()
}
