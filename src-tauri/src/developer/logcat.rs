#[cfg(target_os = "android")]
pub fn start_logcat_listener() {
    std::thread::spawn(|| {
        use std::process::Stdio;
        use tauri::Emitter;

        let Ok(mut child) = std::process::Command::new("logcat")
            .arg("-v")
            .arg("time")
            .stdout(Stdio::piped())
            .spawn()
        else {
            return;
        };

        if let Some(stdout) = child.stdout.take() {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stdout);

            for line in reader.lines().flatten() {
                if let Some(handle) = crate::state::try_app_handle() {
                    let _ = handle.emit(crate::commands::route::LOG, ["ADBLOG", &line]);
                }
            }
        }
    });
}
