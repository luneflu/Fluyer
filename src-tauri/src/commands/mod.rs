pub mod decorum;
pub mod mobile;
pub mod route;

pub const COMMAND_HANDLERS: fn(tauri::ipc::Invoke) -> bool = tauri::generate_handler![
    // Music commands
    crate::music::commands::music_play,
    crate::music::commands::music_pause,
    crate::music::commands::music_next,
    crate::music::commands::music_previous,
    crate::music::commands::music_clear,
    crate::music::commands::music_repeat_mode_set,
    crate::music::commands::music_position_set,
    crate::music::commands::music_queue_add,
    crate::music::commands::music_queue_remove,
    crate::music::commands::music_volume_set,
    crate::music::commands::music_queue_goto,
    crate::music::commands::music_queue_moveto,
    crate::music::commands::music_queue_shuffle,
    crate::music::commands::music_visualizer_buffer_get,
    crate::music::commands::music_image_get,
    // crate::music::commands::music_default_cover_art_get,
    crate::music::commands::music_current_duration_get,
    crate::music::commands::music_player_request_sync,
    crate::music::commands::music_lyrics_get,
    crate::music::commands::music_bit_perfect_toggle,
    #[cfg(desktop)]
    crate::music::commands::music_directory_request,
    #[cfg(desktop)]
    crate::music::commands::music_equalizer,
    #[cfg(desktop)]
    crate::music::commands::music_equalizer_reset,
    // Folder commands
    crate::folder::commands::folder_items_get,
    crate::folder::commands::folder_first_music_path_get,
    // Library commands (Rust-side state, index-based access)
    crate::library::commands::library_load,
    crate::library::commands::library_music_count_get,
    crate::library::commands::library_music_get_by_index,
    crate::library::commands::library_music_get_by_path,
    crate::library::commands::library_album_count_get,
    crate::library::commands::library_album_get_by_index,
    crate::library::commands::library_album_get_first_by_index,
    crate::library::commands::library_folder_info_get,
    crate::library::commands::library_folders_filter_has_music,
    crate::library::commands::music_queue_count_get,
    crate::library::commands::music_queue_get_by_index,
    crate::library::commands::library_collection_add_and_play,
    crate::library::commands::library_collection_add_to_queue,
    crate::library::commands::library_collection_shuffle_and_play,
    crate::library::commands::library_sync,
    // System/Log commands
    crate::system::commands::log_error,
    crate::system::commands::log_info,
    crate::system::commands::update_check,
    #[cfg(target_os = "android")]
    crate::system::commands::toast,
    // Developer commands
    crate::system::commands::developer_log_save,
    crate::system::commands::developer_mpv_log_save,
    // Mobile commands
    #[cfg(target_os = "android")]
    mobile::audio_permission_read_request,
    #[cfg(mobile)]
    mobile::navigation_bar_height_get,
    #[cfg(mobile)]
    mobile::status_bar_height_get,
    #[cfg(mobile)]
    mobile::navigation_bar_visibility_set,
    #[cfg(target_os = "android")]
    mobile::android_directory_request,
    // Cover art commands
    crate::coverart::commands::cover_art_get,
    // Lyric commands
    crate::lyric::commands::lyric_get,
    // Platform-specific commands
    #[cfg(windows)]
    decorum::decorum_show_snap_overlay,
    // Animated Background
    crate::animated_background::animated_background_update,
    crate::animated_background::animated_background_restore,
    // Playlist commands
    crate::playlist::commands::playlist_all_get,
    crate::playlist::commands::playlist_create,
    crate::playlist::commands::playlist_delete,
    crate::playlist::commands::playlist_image_upload,
    crate::playlist::commands::playlist_image_read,
];
