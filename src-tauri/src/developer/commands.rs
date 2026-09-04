use crate::logger;
use crate::state::{app_handle, app_store};
use crate::utils::toast::{Toast, ToastType};
use tauri::Manager;
use tauri_plugin_fluyer::FluyerExt;

#[tauri::command]
pub fn log_error(message: String) {
    crate::error!("{}", message);
}

#[tauri::command]
pub fn log_info(message: String) {
    crate::info!("{}", message);
}

// #[cfg(target_os = "android")]
#[tauri::command]
pub fn toast(message: String) {
    Toast::show(message, ToastType::Info);
}

#[tauri::command]
pub fn developer_log_save() {
    let path = format!(
        "{}/{}",
        app_handle().path().home_dir().unwrap().display(),
        logger::get_log_name()
    );
    std::fs::copy(logger::get_log_path(), path.clone()).unwrap();
    Toast::show(
        format!("Log file saved to {}", path).to_string(),
        ToastType::Info,
    );
}

#[tauri::command]
pub fn developer_mpv_log_save() {
    let path = format!(
        "{}/{}",
        app_handle().path().home_dir().unwrap().display(),
        logger::get_mpv_log_name()
    );
    std::fs::copy(logger::get_mpv_log_path(), path.clone()).unwrap();
    Toast::show(
        format!("Log MPV file saved to {}", path).to_string(),
        ToastType::Info,
    );
}

#[tauri::command]
pub fn developer_clear_data() {
    if let Ok(path) = app_handle().path().app_data_dir() {
        if path.exists() {
            app_store().clear();

            let _ = std::fs::remove_dir_all(&path);
            let _ = std::fs::create_dir_all(&path);
        }
    }
    #[cfg(desktop)]
    app_handle().restart();

    #[cfg(target_os = "android")]
    app_handle().fluyer().restart_app();
}

#[tauri::command]
pub fn developer_clear_cache() {
    if let Ok(path) = app_handle().path().app_cache_dir() {
        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
            let _ = std::fs::create_dir_all(&path);
        }
    }
    #[cfg(desktop)]
    app_handle().restart();

    #[cfg(target_os = "android")]
    app_handle().fluyer().restart_app();
}

#[tauri::command]
pub fn developer_log_get() -> Vec<(String, String)> {
    crate::logger::get_buffered_logs()
}

#[derive(serde::Deserialize)]
struct LatestRelease {
    version: String,
}

fn parse_version(v: &str) -> Vec<u32> {
    v.trim_start_matches('v')
        .split('.')
        .map(|s| s.parse::<u32>().unwrap_or(0))
        .collect()
}

fn is_newer_version(current: &str, latest: &str) -> bool {
    let curr = parse_version(current);
    let late = parse_version(latest);
    let max_len = curr.len().max(late.len());
    for i in 0..max_len {
        let c = curr.get(i).copied().unwrap_or(0);
        let l = late.get(i).copied().unwrap_or(0);
        if l > c {
            return true;
        }
        if c > l {
            return false;
        }
    }
    false
}

/// Check for update using reqwest
#[tauri::command]
pub async fn developer_update_check(current_version: String) -> Result<Option<String>, String> {
    let client = reqwest::Client::builder()
        .user_agent("fluyer-updater")
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get("https://github.com/alvindimas05/Fluyer/releases/latest/download/latest.json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let release: LatestRelease = response.json().await.map_err(|e| e.to_string())?;

    if is_newer_version(&current_version, &release.version) {
        Ok(Some(release.version))
    } else {
        Ok(None)
    }
}

#[derive(serde::Serialize)]
pub struct ProcessMetric {
    pub pid: u32,
    pub name: String,
    pub is_main: bool,
    pub ram_bytes: u64,
    pub working_set_bytes: u64,
    pub private_ws_bytes: u64,
    pub cpu_percent: f32,
    pub gpu_percent: f32,
}

#[derive(serde::Serialize)]
pub struct AppMetrics {
    pub total_ram_bytes: u64,
    pub total_app_ram_bytes: u64,
    pub total_app_working_set_bytes: u64,
    pub total_app_private_ws_bytes: u64,
    pub total_app_cpu_percent: f32,
    pub total_app_gpu_percent: f32,
    pub processes: Vec<ProcessMetric>,
}

#[tauri::command]
pub fn developer_metrics_get() -> AppMetrics {
    #[cfg(target_os = "windows")]
    {
        use std::collections::HashSet;
        use std::sync::Mutex;
        use sysinfo::{Pid, ProcessesToUpdate, System};
        use windows_sys::Win32::System::ProcessStatus::*;
        use windows_sys::Win32::System::Threading::*;

        lazy_static::lazy_static! {
            static ref SYSINFO_SYSTEM: Mutex<System> = Mutex::new(System::new());
        }

        let mut sys_guard = SYSINFO_SYSTEM.lock().unwrap();
        let sys = &mut *sys_guard;
        let main_pid_u32 = std::process::id();
        let main_pid = Pid::from_u32(main_pid_u32);

        sys.refresh_processes(ProcessesToUpdate::All, true);
        sys.refresh_memory();

        let mut all_pids = HashSet::new();
        all_pids.insert(main_pid);

        // Find direct and nested children (WebView2 / msedgewebview2.exe / child processes)
        let mut added = true;
        while added {
            added = false;
            for (&p_pid, process) in sys.processes() {
                if !all_pids.contains(&p_pid) {
                    if let Some(parent) = process.parent() {
                        if all_pids.contains(&parent) {
                            all_pids.insert(p_pid);
                            added = true;
                        }
                    }
                }
            }
        }

        let pid_u32_list: Vec<u32> = all_pids.iter().map(|p| p.as_u32()).collect();
        let gpu_map = get_windows_process_tree_gpu_map(&pid_u32_list).unwrap_or_default();

        let mut processes = Vec::new();
        let mut total_app_ram_bytes = 0u64;
        let mut total_app_working_set_bytes = 0u64;
        let mut total_app_private_ws_bytes = 0u64;
        let mut total_app_cpu_percent = 0.0f32;
        let mut total_app_gpu_percent = 0.0f32;

        #[repr(C)]
        #[allow(non_snake_case)]
        struct PROCESS_MEMORY_COUNTERS_EX2 {
            cb: u32,
            PageFaultCount: u32,
            PeakWorkingSetSize: usize,
            WorkingSetSize: usize,
            QuotaPeakPagedPoolUsage: usize,
            QuotaPagedPoolUsage: usize,
            QuotaPeakNonPagedPoolUsage: usize,
            QuotaNonPagedPoolUsage: usize,
            PagefileUsage: usize,
            PeakPagefileUsage: usize,
            PrivateUsage: usize,
            PrivateWorkingSetSize: usize,
            SharedCommitUsage: u64,
        }

        let cpu_count = sys.cpus().len().max(1) as f32;

        for &pid in &all_pids {
            let pid_u32 = pid.as_u32();
            let mut working_set = 0u64;
            let mut private_usage = 0u64;
            let mut private_ws = 0u64;

            // Direct Win32 memory counters (exact match to Task Manager)
            unsafe {
                let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid_u32);
                if !handle.is_null() {
                    let mut pmc2: PROCESS_MEMORY_COUNTERS_EX2 = std::mem::zeroed();
                    let cb2 = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX2>() as u32;
                    pmc2.cb = cb2;
                    if K32GetProcessMemoryInfo(
                        handle,
                        &mut pmc2 as *mut _ as *mut PROCESS_MEMORY_COUNTERS,
                        cb2,
                    ) != 0
                    {
                        working_set = pmc2.WorkingSetSize as u64;
                        private_usage = pmc2.PrivateUsage as u64;
                        private_ws = pmc2.PrivateWorkingSetSize as u64;
                    } else {
                        // Fallback to EX1
                        let mut pmc: PROCESS_MEMORY_COUNTERS_EX = std::mem::zeroed();
                        let cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
                        pmc.cb = cb;
                        if K32GetProcessMemoryInfo(
                            handle,
                            &mut pmc as *mut _ as *mut PROCESS_MEMORY_COUNTERS,
                            cb,
                        ) != 0
                        {
                            working_set = pmc.WorkingSetSize as u64;
                            private_usage = pmc.PrivateUsage as u64;
                            private_ws = private_usage.min(working_set);
                        }
                    }
                    windows_sys::Win32::Foundation::CloseHandle(handle);
                }
            }

            if let Some(p) = sys.process(pid) {
                let ram = if private_usage > 0 {
                    private_usage
                } else {
                    p.memory()
                };
                let ws = if working_set > 0 {
                    working_set
                } else {
                    p.memory()
                };
                let pws = if private_ws > 0 {
                    private_ws
                } else {
                    ram.min(ws)
                };
                let cpu = p.cpu_usage() / cpu_count;
                let gpu = *gpu_map.get(&pid_u32).unwrap_or(&0.0f32);
                let name = p.name().to_string_lossy().to_string();
                let is_main = pid_u32 == main_pid_u32;

                total_app_ram_bytes += ram;
                total_app_working_set_bytes += ws;
                total_app_private_ws_bytes += pws;
                total_app_cpu_percent += cpu;
                total_app_gpu_percent += gpu;

                processes.push(ProcessMetric {
                    pid: pid_u32,
                    name,
                    is_main,
                    ram_bytes: ram,
                    working_set_bytes: ws,
                    private_ws_bytes: pws,
                    cpu_percent: cpu,
                    gpu_percent: gpu,
                });
            }
        }

        processes.sort_by(|a, b| {
            b.is_main
                .cmp(&a.is_main)
                .then_with(|| b.private_ws_bytes.cmp(&a.private_ws_bytes))
        });

        let total_ram_bytes = sys.total_memory();

        AppMetrics {
            total_ram_bytes,
            total_app_ram_bytes,
            total_app_working_set_bytes,
            total_app_private_ws_bytes,
            total_app_cpu_percent,
            total_app_gpu_percent,
            processes,
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::collections::HashSet;
        use std::sync::Mutex;
        use sysinfo::{Pid, ProcessesToUpdate, System};

        lazy_static::lazy_static! {
            static ref SYSINFO_SYSTEM: Mutex<System> = Mutex::new(System::new());
        }

        let mut sys_guard = SYSINFO_SYSTEM.lock().unwrap();
        let sys = &mut *sys_guard;
        let main_pid_u32 = std::process::id();
        let main_pid = Pid::from_u32(main_pid_u32);

        sys.refresh_processes(ProcessesToUpdate::All, true);
        sys.refresh_memory();

        let mut all_pids = HashSet::new();
        all_pids.insert(main_pid);

        // Find child processes via parent PID hierarchy
        let mut added = true;
        while added {
            added = false;
            for (&p_pid, process) in sys.processes() {
                if !all_pids.contains(&p_pid) {
                    if let Some(parent) = process.parent() {
                        if all_pids.contains(&parent) {
                            all_pids.insert(p_pid);
                            added = true;
                        }
                    }
                }
            }
        }

        // On macOS, query WebKit WKWebView directly for exact WebKit process identifiers
        if let Some(window) = crate::state::app_handle().get_webview_window("main") {
            let (tx, rx) = std::sync::mpsc::channel();
            let _ = window.with_webview(move |webview| {
                extern "C" {
                    fn sel_registerName(name: *const std::ffi::c_char) -> *const std::ffi::c_void;
                    fn objc_msgSend(
                        receiver: *const std::ffi::c_void,
                        op: *const std::ffi::c_void,
                    ) -> i32;
                }

                let mut pids = Vec::new();
                let wk_webview = webview.inner() as *const std::ffi::c_void;
                if !wk_webview.is_null() {
                    unsafe {
                        let sel_web = sel_registerName(b"_webProcessIdentifier\0".as_ptr() as *const _);
                        let sel_gpu = sel_registerName(b"_gpuProcessIdentifier\0".as_ptr() as *const _);
                        let sel_net = sel_registerName(b"_networkProcessIdentifier\0".as_ptr() as *const _);
                        let sel_model = sel_registerName(b"_modelProcessIdentifier\0".as_ptr() as *const _);

                        let web_pid = objc_msgSend(wk_webview, sel_web);
                        let gpu_pid = objc_msgSend(wk_webview, sel_gpu);
                        let net_pid = objc_msgSend(wk_webview, sel_net);
                        let model_pid = objc_msgSend(wk_webview, sel_model);

                        for pid in [web_pid, gpu_pid, net_pid, model_pid] {
                            if pid > 1 {
                                pids.push(pid as u32);
                            }
                        }
                    }
                }
                let _ = tx.send(pids);
            });

            if let Ok(pids) = rx.recv_timeout(std::time::Duration::from_millis(100)) {
                for pid in pids {
                    all_pids.insert(Pid::from_u32(pid));
                }
            }
        }

        extern "C" {
            fn proc_pid_rusage(pid: i32, flavor: i32, buffer: *mut std::ffi::c_void) -> i32;
        }

        #[repr(C)]
        struct RUsageInfoV4 {
            ri_uuid: [u8; 16],
            ri_user_time: u64,
            ri_system_time: u64,
            ri_pkg_idle_wkups: u64,
            ri_interrupt_wkups: u64,
            ri_pageins: u64,
            ri_wired_size: u64,
            ri_resident_size: u64,
            ri_phys_footprint: u64,
        }

        let cpu_count = sys.cpus().len().max(1) as f32;
        let mut processes = Vec::new();
        let mut total_app_ram_bytes = 0u64;
        let mut total_app_cpu_percent = 0.0f32;

        for &pid in &all_pids {
            let pid_u32 = pid.as_u32();
            if let Some(p) = sys.process(pid) {
                // Activity Monitor on macOS reports `phys_footprint` (physical memory footprint)
                let mut ram = p.memory();
                unsafe {
                    let mut ru: RUsageInfoV4 = std::mem::zeroed();
                    if proc_pid_rusage(
                        pid_u32 as i32,
                        4,
                        &mut ru as *mut _ as *mut std::ffi::c_void,
                    ) == 0
                    {
                        if ru.ri_phys_footprint > 0 {
                            ram = ru.ri_phys_footprint;
                        }
                    }
                }

                let cpu = p.cpu_usage() / cpu_count;
                let raw_name = p.name().to_string_lossy();
                let name = if pid_u32 == main_pid_u32 {
                    raw_name.to_string()
                } else if raw_name.contains("WebContent") {
                    "WebContent".to_string()
                } else if raw_name.contains("GPU") {
                    "Graphics and Media".to_string()
                } else if raw_name.contains("Networking") {
                    "Networking".to_string()
                } else {
                    raw_name
                        .trim_start_matches("com.apple.WebKit.")
                        .to_string()
                };
                let is_main = pid_u32 == main_pid_u32;

                total_app_ram_bytes += ram;
                total_app_cpu_percent += cpu;

                processes.push(ProcessMetric {
                    pid: pid_u32,
                    name,
                    is_main,
                    ram_bytes: ram,
                    working_set_bytes: ram,
                    private_ws_bytes: ram,
                    cpu_percent: cpu,
                    gpu_percent: 0.0,
                });
            }
        }

        processes.sort_by(|a, b| {
            b.is_main
                .cmp(&a.is_main)
                .then_with(|| b.ram_bytes.cmp(&a.ram_bytes))
        });

        let total_ram_bytes = sys.total_memory();

        AppMetrics {
            total_ram_bytes,
            total_app_ram_bytes,
            total_app_working_set_bytes: total_app_ram_bytes,
            total_app_private_ws_bytes: total_app_ram_bytes,
            total_app_cpu_percent,
            total_app_gpu_percent: 0.0,
            processes,
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        AppMetrics {
            total_ram_bytes: 0,
            total_app_ram_bytes: 0,
            total_app_working_set_bytes: 0,
            total_app_private_ws_bytes: 0,
            total_app_cpu_percent: 0.0,
            total_app_gpu_percent: 0.0,
            processes: Vec::new(),
        }
    }
}

#[cfg(target_os = "windows")]
fn get_windows_process_tree_gpu_map(
    target_pids: &[u32],
) -> Option<std::collections::HashMap<u32, f32>> {
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::time::Instant;
    use windows_sys::Win32::System::Performance::*;

    struct QueryState {
        query: usize,
        counters: Vec<(usize, u32)>,
        last_check: Instant,
    }

    lazy_static::lazy_static! {
        static ref GPU_QUERY: Mutex<Option<QueryState>> = Mutex::new(None);
    }

    let mut guard = GPU_QUERY.lock().ok()?;

    let now = Instant::now();
    let need_init = match &*guard {
        Some(state) => now.duration_since(state.last_check).as_secs() >= 3,
        None => true,
    };

    if need_init {
        if let Some(old) = guard.take() {
            unsafe { PdhCloseQuery(old.query as PDH_HQUERY) };
        }

        let mut query: PDH_HQUERY = std::ptr::null_mut();
        if unsafe { PdhOpenQueryW(std::ptr::null(), 0, &mut query) } != 0 {
            return None;
        }

        let pid_prefixes: Vec<(String, u32)> = target_pids
            .iter()
            .map(|pid| (format!("pid_{}", pid), *pid))
            .collect();
        let mut counters = Vec::new();

        let object_name: Vec<u16> = "GPU Engine\0".encode_utf16().collect();
        let mut counter_list_size: u32 = 0;
        let mut instance_list_size: u32 = 0;

        unsafe {
            PdhEnumObjectItemsW(
                std::ptr::null(),
                std::ptr::null(),
                object_name.as_ptr(),
                std::ptr::null_mut(),
                &mut counter_list_size,
                std::ptr::null_mut(),
                &mut instance_list_size,
                100,
                0,
            );
        }

        if instance_list_size > 0 {
            let mut instance_buffer: Vec<u16> = vec![0; instance_list_size as usize];
            let mut counter_buffer: Vec<u16> = vec![0; counter_list_size.max(1) as usize];

            let status = unsafe {
                PdhEnumObjectItemsW(
                    std::ptr::null(),
                    std::ptr::null(),
                    object_name.as_ptr(),
                    counter_buffer.as_mut_ptr(),
                    &mut counter_list_size,
                    instance_buffer.as_mut_ptr(),
                    &mut instance_list_size,
                    100,
                    0,
                )
            };

            if status == 0 {
                let mut start = 0;
                for i in 0..instance_buffer.len() {
                    if instance_buffer[i] == 0 {
                        if start < i {
                            let inst_str = String::from_utf16_lossy(&instance_buffer[start..i]);
                            if let Some((_, matched_pid)) = pid_prefixes
                                .iter()
                                .find(|(prefix, _)| inst_str.starts_with(prefix))
                            {
                                let full_path = format!(
                                    "\\GPU Engine({})\\Utilization Percentage\0",
                                    inst_str
                                );
                                let wide_path: Vec<u16> = full_path.encode_utf16().collect();
                                let mut hcounter: PDH_HCOUNTER = std::ptr::null_mut();
                                if unsafe {
                                    PdhAddCounterW(query, wide_path.as_ptr(), 0, &mut hcounter)
                                } == 0
                                {
                                    counters.push((hcounter as usize, *matched_pid));
                                }
                            }
                        }
                        start = i + 1;
                        if start < instance_buffer.len() && instance_buffer[start] == 0 {
                            break;
                        }
                    }
                }
            }
        }

        if unsafe { PdhCollectQueryData(query) } != 0 {
            unsafe { PdhCloseQuery(query) };
            return None;
        }

        *guard = Some(QueryState {
            query: query as usize,
            counters,
            last_check: now,
        });

        return Some(HashMap::new());
    }

    let state = guard.as_mut()?;
    if unsafe { PdhCollectQueryData(state.query as PDH_HQUERY) } != 0 {
        return None;
    }

    let mut gpu_map: HashMap<u32, f32> = HashMap::new();
    for &(counter, pid) in &state.counters {
        let mut display_value = PDH_FMT_COUNTERVALUE {
            CStatus: 0,
            Anonymous: PDH_FMT_COUNTERVALUE_0 { doubleValue: 0.0 },
        };
        if unsafe {
            PdhGetFormattedCounterValue(
                counter as PDH_HCOUNTER,
                0x00000200,
                std::ptr::null_mut(),
                &mut display_value,
            )
        } == 0
        {
            if display_value.CStatus == 0 {
                let val = unsafe { display_value.Anonymous.doubleValue } as f32;
                *gpu_map.entry(pid).or_insert(0.0) += val;
            }
        }
    }

    Some(gpu_map)
}
