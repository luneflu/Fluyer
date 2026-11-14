use super::{FluyerAppEngine, FluyerError};
use crate::services::ytmusic::{AlbumDetail, DeviceLogin, HomeFeed, SearchItem, StreamInfo, TokenStatus};

/// What the confirmation dialog needs: a new profile exists. The payload itself is
/// opaque to the UI and only the core reads it.
#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct ProfileUpdate {
    pub version: u32,
    pub client_version: String,
}

fn failed(reason: String) -> FluyerError {
    FluyerError::ProviderFailed { reason }
}

#[uniffi::export]
impl FluyerAppEngine {
    // ---- YouTube Music ----
    pub async fn provider_search(&self, query: String) -> Result<Vec<SearchItem>, FluyerError> {
        self.inner.provider_search(query).await.map_err(failed)
    }

    /// Home feed shelves, for the anonymous account until a login lands.
    pub async fn provider_home(&self) -> Result<HomeFeed, FluyerError> {
        self.inner.provider_home().await.map_err(failed)
    }

    /// Next home page; each page hands back its own continuation until the feed ends.
    pub async fn provider_home_more(&self, continuation: String) -> Result<HomeFeed, FluyerError> {
        self.inner.provider_home_more(continuation).await.map_err(failed)
    }

    pub async fn provider_album(&self, browse_id: String) -> Result<AlbumDetail, FluyerError> {
        self.inner.provider_album(browse_id).await.map_err(failed)
    }

    pub async fn provider_lyrics(&self, video_id: String) -> Result<String, FluyerError> {
        self.inner.provider_lyrics(video_id).await.map_err(failed)
    }

    /// Formats may still be throttle-protected; the core solves `n` before returning.
    pub async fn provider_stream_formats(&self, video_id: String) -> Result<StreamInfo, FluyerError> {
        self.inner.provider_stream_formats(video_id).await.map_err(failed)
    }

    /// Plays a track by video id; the core resolves + caches the audio and feeds BASS.
    pub async fn provider_play_track(
        &self,
        video_id: String,
        title: String,
        artist: String,
        album: String,
        thumbnail: Option<String>,
        duration_ms: u64,
    ) -> Result<(), FluyerError> {
        self.inner
            .provider_play_track(video_id, title, artist, album, thumbnail, duration_ms)
            .await
            .map_err(failed)
    }

    // ---- login (device code) ----
    pub async fn provider_begin_login(&self) -> Result<DeviceLogin, FluyerError> {
        self.inner.provider_begin_login().await.map_err(failed)
    }

    pub async fn provider_poll_login(&self, device_code: String) -> Result<TokenStatus, FluyerError> {
        self.inner.provider_poll_login(device_code).await.map_err(failed)
    }

    pub fn provider_is_logged_in(&self) -> bool {
        self.inner.provider_is_logged_in()
    }

    pub fn provider_logout(&self) {
        self.inner.provider_logout()
    }

    // ---- profile updates (confirmation required) ----
    pub fn provider_profile_version(&self) -> u32 {
        self.inner.provider_profile_version()
    }

    /// Returns a newer profile without applying it; the UI asks the user first.
    pub async fn provider_check_profile(&self) -> Result<Option<ProfileUpdate>, FluyerError> {
        self.inner
            .provider_check_profile()
            .await
            .map(|pending| {
                pending.map(|profile| ProfileUpdate {
                    version: profile.version,
                    client_version: profile.client_version,
                })
            })
            .map_err(failed)
    }

    pub fn provider_apply_profile(&self) -> bool {
        self.inner.provider_apply_profile()
    }

    /// Remembers "not now": that version is not offered again.
    pub fn provider_dismiss_profile(&self) -> bool {
        self.inner.provider_dismiss_profile()
    }
}
