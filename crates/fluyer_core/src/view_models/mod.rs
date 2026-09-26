pub mod album_card;
pub mod album_detail;
pub mod common;
pub mod formatters;
pub mod play_view;
pub mod player_bar;
pub mod scan_status;
pub mod track_item;

pub use album_card::AlbumCardViewModel;
pub use album_detail::AlbumDetailViewModel;
pub use common::{ColorRgb, LyricLine, NativeRepeatMode};
pub use formatters::{
    find_active_lyric_index, format_album_label, format_duration, format_time, parse_lrc,
    resize_image_rgba,
};
pub use play_view::PlayViewModel;
pub use player_bar::PlayerBarViewModel;
pub use scan_status::ScanStatusViewModel;
pub use track_item::TrackItemViewModel;

// Compatibility aliases for C FFI and existing bindings
pub type TrackItemView = TrackItemViewModel;
pub type AlbumItemView = AlbumCardViewModel;
pub type PlayerBarView = PlayerBarViewModel;
