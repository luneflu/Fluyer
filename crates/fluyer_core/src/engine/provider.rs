//! Provider surface: YouTube Music browse/lyrics/stream plus the profile update flow.
//!
//! Everything here is provider-agnostic from the UI's point of view: one module per
//! provider (`services/ytmusic`), a shared profile and secret store behind them.

use crate::services::profile::Profile;
use crate::services::ytmusic::{
    AlbumDetail, DeviceLogin, HomeFeed, SearchItem, StreamInfo, TokenStatus, YouTubeMusic,
};
use std::sync::Arc;

impl super::FluyerEngine {
    /// Provider instance for the profile currently applied.
    fn provider(&self) -> Arc<YouTubeMusic> {
        let version = self.profile.current().version;
        if let Some((cached, provider)) = self.provider_cache.read().unwrap().as_ref() {
            if *cached == version {
                return Arc::clone(provider);
            }
        }
        let provider = Arc::new(YouTubeMusic::new(
            self.profile.current(),
            Arc::clone(&self.secrets) as Arc<dyn crate::services::SecretStore>,
            &self.data_dir,
        ));
        *self.provider_cache.write().unwrap() = Some((version, Arc::clone(&provider)));
        provider
    }

    // Async variants: uniffi polls these on its own executor, so they must not call
    // `runtime.block_on` (that panics from inside a runtime).
    pub async fn provider_search(&self, query: String) -> Result<Vec<SearchItem>, String> {
        let provider = self.provider();
        self.on_runtime(async move { provider.search(&query).await }).await
    }

    pub async fn provider_home(&self) -> Result<HomeFeed, String> {
        let provider = self.provider();
        self.on_runtime(async move { provider.home().await }).await
    }

    pub async fn provider_home_more(&self, continuation: String) -> Result<HomeFeed, String> {
        let provider = self.provider();
        self.on_runtime(async move { provider.home_more(&continuation).await }).await
    }

    pub async fn provider_album(&self, browse_id: String) -> Result<AlbumDetail, String> {
        let provider = self.provider();
        self.on_runtime(async move { provider.album(&browse_id).await }).await
    }

    pub async fn provider_lyrics(&self, video_id: String) -> Result<String, String> {
        let provider = self.provider();
        self.on_runtime(async move { provider.lyrics(&video_id).await }).await
    }

    pub async fn provider_stream_formats(&self, video_id: String) -> Result<StreamInfo, String> {
        let provider = self.provider();
        self.on_runtime(async move { provider.stream_formats(&video_id).await }).await
    }

    pub async fn provider_play_track(
        &self,
        video_id: String,
        title: String,
        artist: String,
        album: String,
        thumbnail: Option<String>,
        duration_ms: u64,
    ) -> Result<(), String> {
        let info = self.provider_stream_formats(video_id.clone()).await?;
        let (url, expires_in) = pick_audio_url(&info).ok_or("no audio format in the player response")?;
        let path = self.stream_proxy.link(&video_id, &url, expires_in)?;
        if let Some(thumbnail) = thumbnail.as_deref() {
            let cover_art = std::sync::Arc::clone(&self.cover_art);
            let (link, artist, album, title, path) =
                (thumbnail.to_string(), artist.clone(), album.clone(), title.clone(), path.clone());
            if let Err(error) = self
                .on_runtime(async move {
                    cache_remote_cover(&cover_art, &link, &artist, &album, &title, &path).await
                })
                .await
            {
                eprintln!("ytmusic cover skipped: {error}");
            }
        }
        let metadata = crate::metadata::MusicMetadata {
            path,
            title: Some(title),
            artist: Some(artist),
            album: Some(album),
            image: thumbnail,
            duration: Some(duration_ms as u128),
            ..Default::default()
        };
        self.player.clear();
        if !self.player.add_track_no_auto_play(vec![metadata]) {
            return Err("the player rejected the track".to_string());
        }
        self.player.goto_track(0);
        Ok(())
    }

    pub fn provider_search_blocking(
        &self,
        query: String,
    ) -> Result<Vec<SearchItem>, String> {
        let provider = self.provider();
        self.runtime.block_on(provider.search(&query))
    }

    pub fn provider_home_blocking(&self) -> Result<HomeFeed, String> {
        let provider = self.provider();
        self.runtime.block_on(provider.home())
    }

    pub fn provider_home_more_blocking(&self, continuation: String) -> Result<HomeFeed, String> {
        let provider = self.provider();
        self.runtime.block_on(provider.home_more(&continuation))
    }

    pub fn provider_album_blocking(&self, browse_id: String) -> Result<AlbumDetail, String> {
        let provider = self.provider();
        self.runtime.block_on(provider.album(&browse_id))
    }

    pub fn provider_lyrics_blocking(&self, video_id: String) -> Result<String, String> {
        let provider = self.provider();
        self.runtime.block_on(provider.lyrics(&video_id))
    }

    pub fn provider_stream_blocking(&self, video_id: String) -> Result<StreamInfo, String> {
        let provider = self.provider();
        self.runtime.block_on(provider.stream_formats(&video_id))
    }

    pub async fn provider_begin_login(&self) -> Result<DeviceLogin, String> {
        let provider = self.provider();
        self.on_runtime(async move { provider.begin_login().await }).await
    }

    pub async fn provider_poll_login(&self, device_code: String) -> Result<TokenStatus, String> {
        let provider = self.provider();
        self.on_runtime(async move { provider.poll_login(&device_code).await }).await
    }

    pub async fn provider_check_profile(&self) -> Result<Option<Profile>, String> {
        let profile = std::sync::Arc::clone(&self.profile);
        self.on_runtime(async move { profile.check().await }).await
    }

    pub fn provider_begin_login_blocking(&self) -> Result<DeviceLogin, String> {
        let provider = self.provider();
        self.runtime.block_on(provider.begin_login())
    }

    pub fn provider_poll_login_blocking(&self, device_code: String) -> Result<TokenStatus, String> {
        let provider = self.provider();
        self.runtime.block_on(provider.poll_login(&device_code))
    }

    /// Resolves a track and plays it: the URL takes the place of `path`, so the queue,
    /// bar, seek and repeat paths work unchanged.
    pub fn provider_play_track_blocking(
        &self,
        video_id: String,
        title: String,
        artist: String,
        album: String,
        thumbnail: Option<String>,
        duration_ms: u64,
    ) -> Result<(), String> {
        let info = self.provider_stream_blocking(video_id.clone())?;
        let (url, expires_in) = pick_audio_url(&info).ok_or("no audio format in the player response")?;
        let path = self.stream_proxy.link(&video_id, &url, expires_in)?;
        // The provider's own cover is authoritative: store it under the same key the
        // artwork resolver looks up, so no MusicBrainz lookup ever runs for it.
        if let Some(thumbnail) = thumbnail.as_deref() {
            if let Err(error) = self.runtime.block_on(cache_remote_cover(&self.cover_art, thumbnail, &artist, &album, &title, &path)) {
                eprintln!("ytmusic cover skipped: {error}");
            }
        }
        let metadata = crate::metadata::MusicMetadata {
            path,
            title: Some(title),
            artist: Some(artist),
            album: Some(album),
            image: thumbnail,
            duration: Some(duration_ms as u128),
            ..Default::default()
        };
        self.player.clear();
        if !self.player.add_track_no_auto_play(vec![metadata]) {
            return Err("the player rejected the track".to_string());
        }
        self.player.goto_track(0);
        Ok(())
    }

    pub fn provider_is_logged_in(&self) -> bool {
        self.provider().is_logged_in()
    }

    pub fn provider_logout(&self) {
        self.provider().logout();
    }

    /// Version of the profile in use (0 = bundled placeholder, remote calls unusable).
    pub fn provider_profile_version(&self) -> u32 {
        self.profile.current().version
    }

    /// Downloads a newer profile and parks it as pending: the UI must confirm before
    /// [`Self::provider_apply_profile`] makes it current.
    pub fn provider_check_profile_blocking(&self) -> Result<Option<Profile>, String> {
        self.runtime.block_on(self.profile.check())
    }

    pub fn provider_apply_profile(&self) -> bool {
        let applied = self.profile.apply();
        if applied {
            *self.provider_cache.write().unwrap() = None;
        }
        applied
    }

    pub fn provider_dismiss_profile(&self) -> bool {
        self.profile.dismiss()
    }
}

/// Highest-bitrate audio format, AAC/MP4 first: it needs no extra BASS plugin, and the
/// WebM/Opus fallback only when AAC is missing.
fn pick_audio_url(info: &StreamInfo) -> Option<(String, Option<u64>)> {
    let audio = |needle: &str| -> Option<&crate::services::ytmusic::StreamFormat> {
        info.formats
            .iter()
            .filter(|format| format.mime_type.as_deref().is_some_and(|mime| mime.contains("audio") && mime.contains(needle)))
            .filter(|format| format.url.is_some())
            .max_by_key(|format| format.bitrate.unwrap_or(0))
    };
    audio("mp4")
        .or_else(|| audio("webm"))
        .and_then(|format| format.url.clone().map(|url| (url, format.expires_in_seconds)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ytmusic::StreamFormat;

    fn format(mime: &str, bitrate: u64, url: Option<&str>) -> StreamFormat {
        StreamFormat {
            mime_type: Some(mime.to_string()),
            bitrate: Some(bitrate),
            url: url.map(str::to_string),
            signature_cipher: None,
            expires_in_seconds: None,
        }
    }

    #[test]
    fn audio_pick_prefers_aac_then_webm_and_skips_cipher_only() {
        let info = StreamInfo {
            formats: vec![
                format("audio/webm; codecs=\"opus\"", 134_700, Some("webm-best")),
                format("audio/mp4; codecs=\"mp4a.40.2\"", 131_017, Some("aac")),
                format("audio/mp4; codecs=\"mp4a.40.5\"", 50_517, Some("aac-he")),
                format("video/mp4; codecs=\"avc1\"", 900_000, Some("video")),
                format("audio/mp4; codecs=\"mp4a.40.2\"", 999_999, None),
            ],
            playability_status: Some("OK".into()),
            reason: None,
        };
        assert_eq!(pick_audio_url(&info).map(|(url, _)| url).as_deref(), Some("aac"));
    }
}

/// Downloads a provider cover into the artwork disk cache, keyed exactly like
/// `artwork_resolve_track` looks it up (artist, album, title, audio path).
async fn cache_remote_cover(
    cover_art: &crate::services::CoverArtService,
    url: &str,
    artist: &str,
    album: &str,
    title: &str,
    path: &str,
) -> Result<(), String> {
    let response = reqwest::Client::new()
        .get(url)
        .timeout(std::time::Duration::from_secs(20))
        .send()
        .await
        .map_err(|e| format!("cover download failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("cover download returned {}", response.status()));
    }
    let bytes = response.bytes().await.map_err(|e| format!("cover body failed: {e}"))?;
    cover_art
        .save_to_cache(artist, Some(album), Some(title), Some(path), &bytes)
        .map_err(|e| format!("cover cache write: {e}"))
}

