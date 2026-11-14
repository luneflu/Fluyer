//! YouTube Music provider: browse, album pages, lyrics, and streaming descriptors.
//!
//! Scope is deliberately narrow — this is not a full port of ytmusicapi:
//! `search`, `album`, `lyrics`, `stream_formats` and the device-code login.
//! Anything data-shaped (client version, API key, endpoint paths, auth client) comes
//! from `services/profile.rs`, so a rotation is a profile update, not a release.
//!
//! Parsers are Rust (`parsers.rs`), not JS: nothing downloaded is executed. The
//! cipher half of streaming is handed onward as `signatureCipher` — solving it needs
//! Google's player JS and is not part of this module yet.

pub mod auth;
pub mod cipher;
pub mod models;
pub mod models_auth;
pub mod parsers;

pub use models::*;
pub use models_auth::*;

use crate::services::profile::Profile;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const DEFAULT_BASE: &str = "https://music.youtube.com/youtubei/v1";
/// Playback asks `www.youtube.com` with the VISIONOS client: it returns plain
/// `url`s (no `signatureCipher`) and needs no PO token, only a visitor id.
const PLAYER_URL: &str = "https://www.youtube.com/youtubei/v1/player?prettyPrint=false";
const PLAYER_CLIENT: &str = "VISIONOS";
const PLAYER_VERSION: &str = "1.02";
const PLAYER_CLIENT_ID: &str = "101";
const VISIONOS_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 15_7_3) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.0 Safari/605.1.15";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:88.0) Gecko/20100101 Firefox/88.0";

pub struct YouTubeMusic {
    pub(crate) client: reqwest::Client,
    pub(crate) secrets: Arc<dyn crate::services::SecretStore>,
    /// Snapshot of the profile this instance was built with.
    profile: Profile,
    language: String,
    region: String,
    /// `visitorData` from any earlier response; the player call requires one.
    visitor: Mutex<Option<String>>,
    cipher: cipher::Solver,
}

impl YouTubeMusic {
    pub fn new(
        profile: Profile,
        secrets: Arc<dyn crate::services::SecretStore>,
        data_dir: &Path,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent(USER_AGENT)
            .build()
            .unwrap_or_default();
        Self {
            client,
            secrets,
            profile,
            language: "en".into(),
            region: "US".into(),
            visitor: Mutex::new(None),
            cipher: cipher::Solver::new(data_dir),
        }
    }

    pub fn with_locale(mut self, language: &str, region: &str) -> Self {
        self.language = language.to_string();
        self.region = region.to_string();
        self
    }

    pub fn profile(&self) -> Profile {
        self.profile.clone()
    }

    /// Search across songs, albums, artists and playlists.
    pub async fn search(&self, query: &str) -> Result<Vec<SearchItem>, String> {
        let body = json!({ "query": query });
        let response = self.request("search", body).await?;
        Ok(parsers::parse_search(&response))
    }

    /// Home feed: carousel shelves for the anonymous (or signed-in) account.
    pub async fn home(&self) -> Result<HomeFeed, String> {
        let response = self.request("browse", json!({ "browseId": "FEmusic_home" })).await?;
        Ok(parsers::parse_home(&response))
    }

    /// Next home page. Each page carries its own token until the feed ends.
    pub async fn home_more(&self, continuation: &str) -> Result<HomeFeed, String> {
        let response = self.request("browse", json!({ "continuation": continuation })).await?;
        Ok(parsers::parse_home(&response))
    }

    /// Album page: header + tracks. `browse_id` comes from a search hit.
    pub async fn album(&self, browse_id: &str) -> Result<AlbumDetail, String> {
        let body = json!({ "browseId": browse_id });
        let response = self.request("browse", body).await?;
        parsers::parse_album(&response, browse_id).ok_or_else(|| "album not found in response".to_string())
    }

    /// Plain lyric text for a track.
    pub async fn lyrics(&self, video_id: &str) -> Result<String, String> {
        // The lyrics live on a separate browse page whose id is published in `next`.
        let next = self.request("next", json!({ "videoId": video_id })).await?;
        let lyrics_id = find_lyrics_browse_id(&next).ok_or("no lyrics for this track")?;
        let response = self.request("browse", json!({ "browseId": lyrics_id })).await?;
        parsers::parse_lyrics(&response).ok_or_else(|| "lyrics page had no text".to_string())
    }

    /// Streaming formats for a track. URLs may still be cipher-protected; the caller
    /// gets both `url` and `signature_cipher` and decides what to do with them.
    pub async fn stream_formats(&self, video_id: &str) -> Result<StreamInfo, String> {
        let mut response = self.player_response(video_id).await?;
        let status = nav_str(&response, &["playabilityStatus", "status"]).unwrap_or_default();
        if status != "OK" {
            let reason = nav_str(&response, &["playabilityStatus", "reason"]).unwrap_or_default();
            return Err(format!("{status}: {reason}"));
        }
        // Best effort: an unsolved `n` costs speed, not playback, so a solver failure
        // must not hide the formats we already have.
        if let Err(error) = self.cipher.solve_streaming(&self.client, &mut response).await {
            log::debug!("ytmusic: cipher solve skipped: {error}");
        }
        parsers::parse_streaming(&response).ok_or_else(|| "no streaming data in response".to_string())
    }

    /// Prepares the cipher solver and returns `player id / signatureTimestamp`.
    /// Mostly a diagnostic: it proves the vendored EJS solver still matches the player.
    pub async fn cipher_warm(&self) -> Result<(String, u32), String> {
        let player = self.cipher.player(&self.client).await?;
        Ok((player.id, player.signature_timestamp))
    }

    /// Solves a single `n` (throttle) value with the cached player.
    pub async fn cipher_solve_n(&self, n: &str) -> Result<String, String> {
        let player = self.cipher.player(&self.client).await?;
        self.cipher
            .solve(&player, Some(n.to_string()), None)
            .await?
            .n
            .ok_or_else(|| "solver returned no n".to_string())
    }

    /// `player` via the VISIONOS client, carrying the visitor id it needs.
    async fn player_response(&self, video_id: &str) -> Result<Value, String> {
        let visitor = self.visitor_data().await?;
        let body = json!({
            "context": {"client": {
                "clientName": PLAYER_CLIENT,
                "clientVersion": PLAYER_VERSION,
                "deviceMake": "Apple",
                "deviceModel": "RealityDevice17,1",
                "userAgent": VISIONOS_UA,
                "osName": "visionOS",
                "osVersion": "26.5.23O471",
                "hl": self.language,
                "gl": self.region,
                "visitorData": visitor,
            }},
            "videoId": video_id,
            "contentCheckOk": true,
            "racyCheckOk": true,
        });
        let response = self
            .client
            .post(PLAYER_URL)
            .header("User-Agent", VISIONOS_UA)
            .header("X-YouTube-Client-Name", PLAYER_CLIENT_ID)
            .header("X-YouTube-Client-Version", PLAYER_VERSION)
            .header("X-Goog-Visitor-Id", visitor)
            .header("Origin", "https://www.youtube.com")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("player request failed: {e}"))?;
        let status = response.status();
        let text = response.text().await.map_err(|e| format!("player body failed: {e}"))?;
        if !status.is_success() {
            return Err(format!("player returned {status}"));
        }
        let parsed: Value = serde_json::from_str(&text).map_err(|e| format!("player returned non-JSON: {e}"))?;
        self.capture_visitor(&parsed);
        Ok(parsed)
    }

    /// Cached `visitorData`, fetched with a home browse when nothing has supplied one yet.
    async fn visitor_data(&self) -> Result<String, String> {
        if let Some(visitor) = self.visitor.lock().expect("visitor").clone() {
            return Ok(visitor);
        }
        let _ = self.request("browse", json!({ "browseId": "FEmusic_home" })).await;
        self.visitor
            .lock()
            .expect("visitor")
            .clone()
            .ok_or_else(|| "no visitorData from the API".to_string())
    }

    fn capture_visitor(&self, response: &Value) {
        if let Some(visitor) = nav_str(response, &["responseContext", "visitorData"]) {
            *self.visitor.lock().expect("visitor") = Some(visitor);
        }
    }

    /// POSTs an InnerTube endpoint with the profile's envelope, adding the bearer
    /// token when logged in.
    async fn request(&self, endpoint: &str, body: Value) -> Result<Value, String> {
        let response = self
            .send(endpoint, body, self.access_token().await.as_deref())
            .await?;
        self.capture_visitor(&response);
        Ok(response)
    }

    pub(crate) async fn send(
        &self,
        endpoint: &str,
        body: Value,
        token: Option<&str>,
    ) -> Result<Value, String> {
        let url = endpoint_url(self.base(), &self.endpoint_path(endpoint), &self.profile.api_key);
        let mut request = self
            .client
            .post(url)
            .header("X-Origin", "https://music.youtube.com")
            .header("Accept-Language", format!("{}-{}", self.language, self.region))
            .json(&self.envelope(body));
        if let Some(token) = token {
            request = request.header("Authorization", format!("Bearer {token}"));
        }
        let response = request.send().await.map_err(|e| format!("{endpoint} request failed: {e}"))?;
        let status = response.status();
        let text = response.text().await.map_err(|e| format!("{endpoint} body failed: {e}"))?;
        if !status.is_success() {
            return Err(format!("{endpoint} returned {status}"));
        }
        serde_json::from_str(&text).map_err(|e| format!("{endpoint} returned non-JSON: {e}"))
    }

    fn base(&self) -> &str {
        DEFAULT_BASE
    }

    /// Endpoint path from the profile when present, else the known default.
    fn endpoint_path(&self, endpoint: &str) -> String {
        let fallback = match endpoint {
            "search" => "/search".to_string(),
            other => format!("/{other}"),
        };
        self.profile
            .endpoints
            .get(endpoint)
            .cloned()
            .unwrap_or(fallback)
    }

    fn envelope(&self, mut body: Value) -> Value {
        let Some(map) = body.as_object_mut() else { return body };
        map.insert(
            "context".to_string(),
            json!({
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": self.profile.client_version,
                    "hl": self.language,
                    "gl": self.region,
                },
                "user": { "lockedSafetyMode": false },
            }),
        );
        // Home continuation tokens are bound to the visitor that minted them; without it
        // the server answers an empty page.
        if let Some(visitor) = self.visitor.lock().expect("visitor").clone() {
            if let Some(client) = map
                .get_mut("context")
                .and_then(|context| context.get_mut("client"))
                .and_then(Value::as_object_mut)
            {
                client.insert("visitorData".to_string(), Value::String(visitor));
            }
        }
        body
    }
}

/// `browseId` of the lyrics page, found among the `next` response's browse endpoints.
/// `{base}{path}` with exactly one separator, plus the InnerTube query params.
pub fn endpoint_url(base: &str, path: &str, api_key: &str) -> String {
    format!(
        "{}/{}?alt=json&prettyPrint=false&key={}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/'),
        api_key
    )
}

pub fn find_lyrics_browse_id(next: &Value) -> Option<String> {
    fn walk(value: &Value) -> Option<String> {
        match value {
            Value::Object(map) => {
                if let Some(endpoint) = map.get("browseEndpoint") {
                    let id = endpoint.get("browseId").and_then(Value::as_str);
                    let page_type = endpoint
                        .get("browseEndpointContextSupportedConfigs")
                        .and_then(|c| c.get("browseEndpointContextMusicConfig"))
                        .and_then(|c| c.get("pageType"))
                        .and_then(Value::as_str);
                    if matches!(page_type, Some("MUSIC_PAGE_TYPE_TRACK_LYRICS")) {
                        if let Some(id) = id {
                            return Some(id.to_string());
                        }
                    }
                }
                map.values().find_map(walk)
            }
            Value::Array(items) => items.iter().find_map(walk),
            _ => None,
        }
    }
    walk(next)
}

pub(crate) fn nav_str(value: &Value, path: &[&str]) -> Option<String> {
    parsers::nav(value, path).and_then(Value::as_str).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::Secrets;
    use serde_json::json;

    fn provider() -> YouTubeMusic {
        let dir = std::env::temp_dir().join(format!("fluyer-ytm-{}", std::process::id()));
        YouTubeMusic::new(
            Profile {
                version: 1,
                client_version: "1.20250101.01.00".into(),
                api_key: "KEY".into(),
                endpoints: [("search".to_string(), "/search".to_string())].into(),
                ..Profile::default()
            },
            Arc::new(Secrets::memory()),
            &dir,
        )
    }

    #[test]
    fn envelope_carries_client_context_and_keeps_body_fields() {
        let music = provider();
        let body = music.envelope(json!({ "query": "dabin" }));
        assert_eq!(nav_str(&body, &["query"]).as_deref(), Some("dabin"));
        assert_eq!(nav_str(&body, &["context", "client", "clientVersion"]).as_deref(), Some("1.20250101.01.00"));
        assert_eq!(nav_str(&body, &["context", "client", "clientName"]).as_deref(), Some("WEB_REMIX"));
    }

    #[test]
    fn url_has_exactly_one_separator_and_carries_the_key() {
        assert_eq!(
            endpoint_url("https://music.youtube.com/youtubei/v1", "/search", "KEY"),
            "https://music.youtube.com/youtubei/v1/search?alt=json&prettyPrint=false&key=KEY"
        );
        assert_eq!(endpoint_url("https://h/v1/", "search", "K"), "https://h/v1/search?alt=json&prettyPrint=false&key=K");
    }

    #[test]
    fn endpoint_path_prefers_the_profile_and_falls_back() {
        let music = provider();
        assert_eq!(music.endpoint_path("search"), "/search");
        assert_eq!(music.endpoint_path("player"), "/player");

        let mut custom = provider();
        custom.profile.endpoints.insert("browse".into(), "/v2/browse".into());
        assert_eq!(custom.endpoint_path("browse"), "/v2/browse");
    }

    #[test]
    fn lyrics_page_id_is_read_from_a_page_type_marker() {
        let next = json!({"contents": {"a": [
            {"navigationEndpoint": {"browseEndpoint": {"browseId": "MPLYt_1",
                "browseEndpointContextSupportedConfigs":
                    {"browseEndpointContextMusicConfig": {"pageType": "MUSIC_PAGE_TYPE_TRACK_LYRICS"}}}}}
        ]}});
        assert_eq!(find_lyrics_browse_id(&next).as_deref(), Some("MPLYt_1"));
        assert_eq!(find_lyrics_browse_id(&json!({"x": 1})), None);
    }

    #[test]
    fn logged_out_provider_has_no_token() {
        let music = provider();
        assert!(!music.is_logged_in());
        assert!(music.profile().endpoints.contains_key("search"));
    }
}
