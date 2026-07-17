pub const MUSIC_PLAYER_SYNC: &str = "music_player_sync";
#[cfg(desktop)]
pub const MUSIC_DIRECTORY_REQUEST: &str = "music_directory_request";
pub const ANIMATED_BACKGROUND_TRANSITION_COMPLETE: &str = "animated_background_transition_complete";
#[cfg(target_os = "linux")]
pub const SIDEBAR_MOUSE_LEAVE: &str = "sidebar_mouse_leave";
// pub const AUDIO_PERMISSION_READ_CHECK: &str = "audio_permission_read_check";
// pub const AUDIO_PERMISSION_READ_REQUEST: &str = "audio_permission_read_request";
// #[cfg(target_os = "android")]
// pub const MUSIC_HEADSET_CHANGE: &str = "music_headset_change";
// pub const MUSIC_QUEUE_REMOVE: &str = "music_queue_remove";
#[cfg(target_os = "android")]
pub const ANDROID_DIRECTORY_REQUEST: &str = "android_directory_request";
pub const LOG: &str = "log";
