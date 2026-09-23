pub mod background;
pub mod coverart;
pub mod discord;
pub mod lyric;
pub mod musicbrainz;

pub use background::{extract_prominent_from_bytes, generate_blurred_background};
pub use coverart::CoverArtService;
pub use discord::DiscordRpc;
pub use lyric::LyricService;
