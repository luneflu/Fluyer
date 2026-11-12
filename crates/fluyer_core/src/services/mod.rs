pub mod background;
pub mod coverart;
pub mod discord;
pub mod lyric;
pub mod musicbrainz;
pub mod settings;

pub use background::{extract_prominent_from_bytes, generate_blurred_background};
pub use coverart::CoverArtService;
pub use discord::DiscordRpc;
pub use lyric::LyricService;
pub use settings::{canonical as canonical_settings, SettingsService};
