pub const BASS_PLUGINS: [&str; 6] = [
    "bassflac", "bassopus", "bassape", "bassalac", "basswv", "bass_aac",
];

pub const BASS_SAMPLE_FLOAT: u32 = 0x100;
pub const BASS_STREAM_DECODE: u32 = 0x200000;
pub const BASS_MIXER_NORAMPIN: u32 = 0x800000;
pub const BASS_ACTIVE_STOPPED: u32 = 0;
pub const BASS_ACTIVE_PLAYING: u32 = 1;
#[allow(dead_code)]
pub const BASS_ACTIVE_PAUSED: u32 = 3;
pub const BASS_POS_BYTE: u32 = 0;
pub const BASS_ATTRIB_VOL: u32 = 2;

pub const BASS_SYNC_END: u32 = 2;
pub const BASS_SYNC_FREE: u32 = 0x10000;
pub const BASS_SYNC_MIXTIME: u32 = 0x40000000;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BASS_DEVICEINFO {
    pub name: *const std::ffi::c_char,
    pub driver: *const std::ffi::c_char,
    pub flags: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BASS_INFO {
    pub flags: u32,
    pub hwsize: u32,
    pub hwfree: u32,
    pub freesam: u32,
    pub free3d: u32,
    pub minrate: u32,
    pub maxrate: u32,
    pub eax: u32,
    pub minbuf: u32,
    pub dsver: u32,
    pub latency: u32,
    pub initflags: u32,
    pub speakers: u32,
    pub freq: u32,
}

#[cfg(desktop)]
#[link(name = "bass")]
#[link(name = "bassmix")]
extern "C" {
    pub fn BASS_Init(
        device: i32,
        freq: u32,
        flags: u32,
        win: *mut std::ffi::c_void,
        clsid: *mut std::ffi::c_void,
    ) -> i32;
    pub fn BASS_GetDeviceInfo(device: u32, info: *mut BASS_DEVICEINFO) -> u32;
    pub fn BASS_GetInfo(info: *mut BASS_INFO) -> u32;
    pub fn BASS_PluginLoad(file: *const std::ffi::c_char, flags: u32) -> u32;
    #[allow(dead_code)]
    pub fn BASS_PluginFree(handle: u32) -> i32;
    pub fn BASS_StreamCreateFile(
        mem: bool,
        file: *const std::ffi::c_void,
        offset: u64,
        length: u64,
        flags: u32,
    ) -> u32;
    pub fn BASS_Mixer_StreamCreate(freq: u32, chans: u32, flags: u32) -> u32;
    pub fn BASS_Mixer_StreamAddChannel(handle: u32, channel: u32, flags: u32) -> i32;
    pub fn BASS_Mixer_ChannelRemove(handle: u32) -> u32;
    #[allow(dead_code)]
    pub fn BASS_Mixer_ChannelIsActive(handle: u32) -> u32;
    pub fn BASS_StreamFree(handle: u32) -> i32;
    pub fn BASS_ChannelPlay(handle: u32, restart: i32) -> i32;
    pub fn BASS_ChannelPause(handle: u32) -> i32;
    pub fn BASS_ChannelStop(handle: u32) -> i32;
    pub fn BASS_ChannelIsActive(handle: u32) -> u32;
    #[allow(dead_code)]
    pub fn BASS_ChannelGetLength(handle: u32, mode: u32) -> u64;
    pub fn BASS_ChannelGetPosition(handle: u32, mode: u32) -> u64;
    pub fn BASS_ChannelSetPosition(handle: u32, pos: u64, mode: u32) -> i32;
    pub fn BASS_ChannelBytes2Seconds(handle: u32, pos: u64) -> f64;
    pub fn BASS_ChannelSeconds2Bytes(handle: u32, pos: f64) -> u64;
    pub fn BASS_ChannelSetAttribute(handle: u32, attrib: u32, value: f32) -> i32;
    #[allow(dead_code)]
    pub fn BASS_ChannelGetAttribute(handle: u32, attrib: u32, value: *mut f32) -> i32;
    pub fn BASS_ErrorGetCode() -> i32;
    pub fn BASS_Free() -> i32;
    pub fn BASS_ChannelSetSync(
        handle: u32,
        type_: u32,
        param: u64,
        proc_: Option<
            unsafe extern "C" fn(handle: u32, channel: u32, data: u32, user: *mut std::ffi::c_void),
        >,
        user: *mut std::ffi::c_void,
    ) -> u32;
}

// Android BASS library loaded dynamically
#[cfg(target_os = "android")]
pub mod bass_android {
    use libloading::{Library, Symbol};
    use std::ffi::c_void;
    use std::sync::OnceLock;

    pub struct BassLibrary {
        _bass: Library,
        _bassmix: Library,
        // BASS functions
        pub bass_init: unsafe extern "C" fn(i32, u32, u32, *mut c_void, *mut c_void) -> i32,
        pub bass_plugin_load: unsafe extern "C" fn(*const i8, u32) -> u32,
        pub bass_stream_create_file:
            unsafe extern "C" fn(bool, *const c_void, u64, u64, u32) -> u32,
        pub bass_stream_free: unsafe extern "C" fn(u32) -> i32,
        pub bass_channel_play: unsafe extern "C" fn(u32, i32) -> i32,
        pub bass_channel_pause: unsafe extern "C" fn(u32) -> i32,
        pub bass_channel_stop: unsafe extern "C" fn(u32) -> i32,
        pub bass_channel_is_active: unsafe extern "C" fn(u32) -> u32,
        pub bass_channel_get_position: unsafe extern "C" fn(u32, u32) -> u64,
        pub bass_channel_set_position: unsafe extern "C" fn(u32, u64, u32) -> i32,
        pub bass_channel_bytes2seconds: unsafe extern "C" fn(u32, u64) -> f64,
        pub bass_channel_seconds2bytes: unsafe extern "C" fn(u32, f64) -> u64,
        pub bass_channel_set_attribute: unsafe extern "C" fn(u32, u32, f32) -> i32,
        pub bass_error_get_code: unsafe extern "C" fn() -> i32,
        pub bass_free: unsafe extern "C" fn() -> i32,
        pub bass_channel_set_sync: unsafe extern "C" fn(
            u32,
            u32,
            u64,
            Option<unsafe extern "C" fn(u32, u32, u32, *mut std::ffi::c_void)>,
            *mut std::ffi::c_void,
        ) -> u32,
        // BASSMIX functions
        pub bass_mixer_stream_create: unsafe extern "C" fn(u32, u32, u32) -> u32,
        pub bass_mixer_stream_add_channel: unsafe extern "C" fn(u32, u32, u32) -> i32,
        pub bass_mixer_channel_remove: unsafe extern "C" fn(u32) -> u32,
    }

    unsafe impl Send for BassLibrary {}
    unsafe impl Sync for BassLibrary {}

    static BASS_LIB: OnceLock<BassLibrary> = OnceLock::new();

    pub fn get_bass() -> Option<&'static BassLibrary> {
        BASS_LIB.get()
    }

    pub fn initialize_bass() -> Result<(), String> {
        if BASS_LIB.get().is_some() {
            return Ok(());
        }

        unsafe {
            let bass = Library::new("libbass.so")
                .map_err(|e| format!("Failed to load libbass.so: {}", e))?;
            let bassmix = Library::new("libbassmix.so")
                .map_err(|e| format!("Failed to load libbassmix.so: {}", e))?;

            // Load BASS functions - extract raw function pointers before moving libraries
            let bass_init_fn: unsafe extern "C" fn(i32, u32, u32, *mut c_void, *mut c_void) -> i32 =
                *bass
                    .get::<unsafe extern "C" fn(i32, u32, u32, *mut c_void, *mut c_void) -> i32>(
                        b"BASS_Init",
                    )
                    .map_err(|e| format!("Failed to load BASS_Init: {}", e))?;
            let bass_plugin_load_fn: unsafe extern "C" fn(*const i8, u32) -> u32 = *bass
                .get::<unsafe extern "C" fn(*const i8, u32) -> u32>(b"BASS_PluginLoad")
                .map_err(|e| format!("Failed to load BASS_PluginLoad: {}", e))?;
            let bass_stream_create_file_fn: unsafe extern "C" fn(
                bool,
                *const c_void,
                u64,
                u64,
                u32,
            ) -> u32 = *bass
                .get::<unsafe extern "C" fn(bool, *const c_void, u64, u64, u32) -> u32>(
                    b"BASS_StreamCreateFile",
                )
                .map_err(|e| format!("Failed to load BASS_StreamCreateFile: {}", e))?;
            let bass_stream_free_fn: unsafe extern "C" fn(u32) -> i32 = *bass
                .get::<unsafe extern "C" fn(u32) -> i32>(b"BASS_StreamFree")
                .map_err(|e| format!("Failed to load BASS_StreamFree: {}", e))?;
            let bass_channel_play_fn: unsafe extern "C" fn(u32, i32) -> i32 = *bass
                .get::<unsafe extern "C" fn(u32, i32) -> i32>(b"BASS_ChannelPlay")
                .map_err(|e| format!("Failed to load BASS_ChannelPlay: {}", e))?;
            let bass_channel_pause_fn: unsafe extern "C" fn(u32) -> i32 = *bass
                .get::<unsafe extern "C" fn(u32) -> i32>(b"BASS_ChannelPause")
                .map_err(|e| format!("Failed to load BASS_ChannelPause: {}", e))?;
            let bass_channel_stop_fn: unsafe extern "C" fn(u32) -> i32 = *bass
                .get::<unsafe extern "C" fn(u32) -> i32>(b"BASS_ChannelStop")
                .map_err(|e| format!("Failed to load BASS_ChannelStop: {}", e))?;
            let bass_channel_is_active_fn: unsafe extern "C" fn(u32) -> u32 = *bass
                .get::<unsafe extern "C" fn(u32) -> u32>(b"BASS_ChannelIsActive")
                .map_err(|e| format!("Failed to load BASS_ChannelIsActive: {}", e))?;
            let bass_channel_get_position_fn: unsafe extern "C" fn(u32, u32) -> u64 = *bass
                .get::<unsafe extern "C" fn(u32, u32) -> u64>(b"BASS_ChannelGetPosition")
                .map_err(|e| format!("Failed to load BASS_ChannelGetPosition: {}", e))?;
            let bass_channel_set_position_fn: unsafe extern "C" fn(u32, u64, u32) -> i32 = *bass
                .get::<unsafe extern "C" fn(u32, u64, u32) -> i32>(b"BASS_ChannelSetPosition")
                .map_err(|e| format!("Failed to load BASS_ChannelSetPosition: {}", e))?;
            let bass_channel_bytes2seconds_fn: unsafe extern "C" fn(u32, u64) -> f64 = *bass
                .get::<unsafe extern "C" fn(u32, u64) -> f64>(b"BASS_ChannelBytes2Seconds")
                .map_err(|e| format!("Failed to load BASS_ChannelBytes2Seconds: {}", e))?;
            let bass_channel_seconds2bytes_fn: unsafe extern "C" fn(u32, f64) -> u64 = *bass
                .get::<unsafe extern "C" fn(u32, f64) -> u64>(b"BASS_ChannelSeconds2Bytes")
                .map_err(|e| format!("Failed to load BASS_ChannelSeconds2Bytes: {}", e))?;
            let bass_channel_set_attribute_fn: unsafe extern "C" fn(u32, u32, f32) -> i32 = *bass
                .get::<unsafe extern "C" fn(u32, u32, f32) -> i32>(b"BASS_ChannelSetAttribute")
                .map_err(|e| format!("Failed to load BASS_ChannelSetAttribute: {}", e))?;
            let bass_error_get_code_fn: unsafe extern "C" fn() -> i32 = *bass
                .get::<unsafe extern "C" fn() -> i32>(b"BASS_ErrorGetCode")
                .map_err(|e| format!("Failed to load BASS_ErrorGetCode: {}", e))?;
            let bass_free_fn: unsafe extern "C" fn() -> i32 = *bass
                .get::<unsafe extern "C" fn() -> i32>(b"BASS_Free")
                .map_err(|e| format!("Failed to load BASS_Free: {}", e))?;
            let bass_channel_set_sync_fn: unsafe extern "C" fn(
                u32,
                u32,
                u64,
                Option<unsafe extern "C" fn(u32, u32, u32, *mut std::ffi::c_void)>,
                *mut std::ffi::c_void,
            ) -> u32 = *bass
                .get::<unsafe extern "C" fn(
                    u32,
                    u32,
                    u64,
                    Option<unsafe extern "C" fn(u32, u32, u32, *mut std::ffi::c_void)>,
                    *mut std::ffi::c_void,
                ) -> u32>(b"BASS_ChannelSetSync")
                .map_err(|e| format!("Failed to load BASS_ChannelSetSync: {}", e))?;

            // Load BASSMIX functions
            let bass_mixer_stream_create_fn: unsafe extern "C" fn(u32, u32, u32) -> u32 = *bassmix
                .get::<unsafe extern "C" fn(u32, u32, u32) -> u32>(b"BASS_Mixer_StreamCreate")
                .map_err(|e| format!("Failed to load BASS_Mixer_StreamCreate: {}", e))?;
            let bass_mixer_stream_add_channel_fn: unsafe extern "C" fn(u32, u32, u32) -> i32 =
                *bassmix
                    .get::<unsafe extern "C" fn(u32, u32, u32) -> i32>(
                        b"BASS_Mixer_StreamAddChannel",
                    )
                    .map_err(|e| format!("Failed to load BASS_Mixer_StreamAddChannel: {}", e))?;
            let bass_mixer_channel_remove_fn: unsafe extern "C" fn(u32) -> u32 = *bassmix
                .get::<unsafe extern "C" fn(u32) -> u32>(b"BASS_Mixer_ChannelRemove")
                .map_err(|e| format!("Failed to load BASS_Mixer_ChannelRemove: {}", e))?;

            let lib = BassLibrary {
                _bass: bass,
                _bassmix: bassmix,
                bass_init: bass_init_fn,
                bass_plugin_load: bass_plugin_load_fn,
                bass_stream_create_file: bass_stream_create_file_fn,
                bass_stream_free: bass_stream_free_fn,
                bass_channel_play: bass_channel_play_fn,
                bass_channel_pause: bass_channel_pause_fn,
                bass_channel_stop: bass_channel_stop_fn,
                bass_channel_is_active: bass_channel_is_active_fn,
                bass_channel_get_position: bass_channel_get_position_fn,
                bass_channel_set_position: bass_channel_set_position_fn,
                bass_channel_bytes2seconds: bass_channel_bytes2seconds_fn,
                bass_channel_seconds2bytes: bass_channel_seconds2bytes_fn,
                bass_channel_set_attribute: bass_channel_set_attribute_fn,
                bass_error_get_code: bass_error_get_code_fn,
                bass_free: bass_free_fn,
                bass_channel_set_sync: bass_channel_set_sync_fn,
                bass_mixer_stream_create: bass_mixer_stream_create_fn,
                bass_mixer_stream_add_channel: bass_mixer_stream_add_channel_fn,
                bass_mixer_channel_remove: bass_mixer_channel_remove_fn,
            };

            BASS_LIB
                .set(lib)
                .map_err(|_| "Failed to set BASS library")?;
            Ok(())
        }
    }
}
