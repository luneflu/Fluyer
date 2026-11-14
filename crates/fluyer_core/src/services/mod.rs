pub mod background;
pub mod coverart;
pub mod discord;
pub mod lyric;
pub mod musicbrainz;
pub mod profile;
pub mod secrets;
pub mod settings;
pub mod stream_proxy;
pub mod ytmusic;

pub use background::{extract_prominent_from_bytes, generate_blurred_background};
pub use coverart::CoverArtService;
pub use discord::DiscordRpc;
pub use lyric::LyricService;
pub use profile::{Profile, ProfileService};
pub use secrets::{SecretStore, Secrets};
pub use settings::{canonical as canonical_settings, SettingsService};
pub use stream_proxy::StreamProxy;
