#[macro_export]
macro_rules! flog {
    ($tag:expr, $($arg:tt)*) => {{
        #[cfg(target_os = "android")]
        $crate::logger::android_log(4, $tag, format!($($arg)*));
        #[cfg(not(target_os = "android"))]
        eprintln!("\x1b[1;36m[{}]\x1b[0m \x1b[32m{}\x1b[0m", $tag, format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! flog_err {
    ($tag:expr, $($arg:tt)*) => {{
        #[cfg(target_os = "android")]
        $crate::logger::android_log(6, $tag, format!($($arg)*));
        #[cfg(not(target_os = "android"))]
        eprintln!("\x1b[1;31m[{}]\x1b[0m \x1b[31m{}\x1b[0m", $tag, format_args!($($arg)*));
    }};
}

/// stderr is discarded on Android, so `flog!` and the `log` crate go to logcat
/// (`adb logcat -s Fluyer`). `prio` is the NDK `android_LogPriority`.
#[cfg(target_os = "android")]
pub fn android_log(prio: i32, tag: &str, msg: String) {
    use std::ffi::{c_char, CString};
    #[link(name = "log")]
    extern "C" {
        fn __android_log_write(prio: i32, tag: *const c_char, text: *const c_char) -> i32;
    }
    let text = CString::new(format!("[{}] {}", tag, msg).replace('\0', " ")).unwrap_or_default();
    unsafe { __android_log_write(prio, c"Fluyer".as_ptr(), text.as_ptr()) };
}

#[cfg(target_os = "android")]
struct Logcat;

#[cfg(target_os = "android")]
impl log::Log for Logcat {
    fn enabled(&self, m: &log::Metadata) -> bool {
        m.level() <= log::Level::Info
    }
    fn log(&self, r: &log::Record) {
        if self.enabled(r.metadata()) {
            let prio = match r.level() {
                log::Level::Error => 6,
                log::Level::Warn => 5,
                _ => 4,
            };
            android_log(prio, r.target(), r.args().to_string());
        }
    }
    fn flush(&self) {}
}

/// Routes `log::` records to logcat; no-op elsewhere (desktop has no logger installed).
pub fn init() {
    #[cfg(target_os = "android")]
    if log::set_logger(&Logcat).is_ok() {
        log::set_max_level(log::LevelFilter::Info);
    }
}
