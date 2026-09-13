pub const BASS_PLUGINS: [&str; 4] = ["bassflac", "bassopus", "bassape", "basswv"];

pub const BASS_UNICODE: u32 = 0x80000000;
pub const BASS_SAMPLE_FLOAT: u32 = 0x100;
pub const BASS_STREAM_DECODE: u32 = 0x200000;
pub const BASS_MIXER_NORAMPIN: u32 = 0x800000;
pub const BASS_ACTIVE_STOPPED: u32 = 0;
pub const BASS_ACTIVE_PLAYING: u32 = 1;
pub const BASS_ACTIVE_STALLED: u32 = 2;
pub const BASS_ACTIVE_PAUSED: u32 = 3;
pub const BASS_POS_BYTE: u32 = 0;
pub const BASS_ATTRIB_VOL: u32 = 2;

pub const BASS_MIXER_NONSTOP: u32 = 0x20000;
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
    pub fn BASS_PluginFree(handle: u32) -> i32;
    pub fn BASS_StreamCreateFile(
        mem: i32,
        file: *const std::ffi::c_void,
        offset: u64,
        length: u64,
        flags: u32,
    ) -> u32;
    pub fn BASS_Mixer_StreamCreate(freq: u32, chans: u32, flags: u32) -> u32;
    pub fn BASS_Mixer_StreamAddChannel(handle: u32, channel: u32, flags: u32) -> i32;
    pub fn BASS_Mixer_ChannelRemove(handle: u32) -> u32;
    pub fn BASS_Mixer_ChannelIsActive(handle: u32) -> u32;
    pub fn BASS_StreamFree(handle: u32) -> i32;
    pub fn BASS_ChannelPlay(handle: u32, restart: i32) -> i32;
    pub fn BASS_ChannelPause(handle: u32) -> i32;
    pub fn BASS_ChannelStop(handle: u32) -> i32;
    pub fn BASS_ChannelIsActive(handle: u32) -> u32;
    pub fn BASS_ChannelGetLength(handle: u32, mode: u32) -> u64;
    pub fn BASS_ChannelGetPosition(handle: u32, mode: u32) -> u64;
    pub fn BASS_ChannelSetPosition(handle: u32, pos: u64, mode: u32) -> i32;
    pub fn BASS_ChannelBytes2Seconds(handle: u32, pos: u64) -> f64;
    pub fn BASS_ChannelSeconds2Bytes(handle: u32, pos: f64) -> u64;
    pub fn BASS_ChannelSetAttribute(handle: u32, attrib: u32, value: f32) -> i32;
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
