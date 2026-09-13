#[macro_export]
macro_rules! flog {
    ($tag:expr, $($arg:tt)*) => {
        eprintln!("\x1b[1;36m[{}]\x1b[0m \x1b[32m{}\x1b[0m", $tag, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! flog_err {
    ($tag:expr, $($arg:tt)*) => {
        eprintln!("\x1b[1;31m[{}]\x1b[0m \x1b[31m{}\x1b[0m", $tag, format_args!($($arg)*))
    };
}
