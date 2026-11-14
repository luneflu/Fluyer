//! Device-code login (the flow this app ships) and token refresh.
//!
//! Google's TV/device client ids rotate and are not ours to publish, so they travel in
//! the updatable profile (`services/profile.rs`) rather than being compiled in.
//! Tokens go to `SecretStore`, never to `settings.json`.

use super::models_auth::{DeviceLogin, TokenStatus};
use super::YouTubeMusic;
use crate::services::profile::Profile;
use serde_json::Value;

pub const SCOPE: &str = "https://www.googleapis.com/auth/youtube";
pub const CODE_URL: &str = "https://www.youtube.com/o/oauth2/device/code";
pub const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const TOKEN_KEY: &str = "ytmusic.oauth";
/// ytmusicapi appends this to the UA for device code endpoints; Google accepts it.
const AUTH_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:88.0) Gecko/20100101 Firefox/88.0 Cobalt/Version";

impl YouTubeMusic {
    /// First step: ask Google for a user code. The UI shows `user_code` and
    /// `verification_url`, then calls [`Self::poll_login`] until it returns a token.
    pub async fn begin_login(&self) -> Result<DeviceLogin, String> {
        let (client_id, _) = self.auth_client().ok_or("profile has no auth client")?;
        let response = self
            .client
            .post(CODE_URL)
            .header("User-Agent", AUTH_USER_AGENT)
            .form(&[("scope", SCOPE), ("client_id", client_id.as_str())])
            .send()
            .await
            .map_err(|e| format!("device code request failed: {e}"))?;
        parse_device_code(&json_body(response).await?)
    }

    /// Poll one step. `TokenStatus::Pending` means "ask again after `interval`"; a
    /// token is stored in the secret store before returning.
    pub async fn poll_login(&self, device_code: &str) -> Result<TokenStatus, String> {
        let (client_id, client_secret) = self.auth_client().ok_or("profile has no auth client")?;
        let response = self
            .client
            .post(TOKEN_URL)
            .header("User-Agent", AUTH_USER_AGENT)
            .form(&[
                ("client_id", client_id.as_str()),
                ("client_secret", client_secret.as_str()),
                ("grant_type", "http://oauth.net/grant_type/device/1.0"),
                ("code", device_code),
            ])
            .send()
            .await
            .map_err(|e| format!("token request failed: {e}"))?;
        let status = parse_token(response.status().as_u16(), &json_body(response).await?)?;
        if let TokenStatus::Granted { access_token, refresh_token, expires_in } = &status {
            self.store_token(access_token, refresh_token.as_deref(), *expires_in);
        }
        Ok(status)
    }

    /// Bearer token for request headers; refreshes when expired.
    pub async fn access_token(&self) -> Option<String> {
        let stored: StoredToken = serde_json::from_str(&self.secrets.get(TOKEN_KEY)?).ok()?;
        let now = now_secs();
        if stored.expires_at > now + 60 {
            return Some(stored.access_token);
        }
        let refresh = stored.refresh_token?;
        self.refresh_token(&refresh).await
    }

    pub fn is_logged_in(&self) -> bool {
        self.secrets.get(TOKEN_KEY).is_some()
    }

    pub fn logout(&self) {
        self.secrets.delete(TOKEN_KEY);
    }

    fn store_token(&self, access_token: &str, refresh_token: Option<&str>, expires_in: u64) {
        let previous: Option<StoredToken> = self
            .secrets
            .get(TOKEN_KEY)
            .and_then(|raw| serde_json::from_str(&raw).ok());
        let token = StoredToken {
            access_token: access_token.to_string(),
            refresh_token: refresh_token.map(str::to_string).or(previous.and_then(|p| p.refresh_token)),
            expires_at: now_secs() + expires_in,
        };
        if let Ok(json) = serde_json::to_string(&token) {
            self.secrets.set(TOKEN_KEY, &json);
        }
    }

    async fn refresh_token(&self, refresh_token: &str) -> Option<String> {
        let (client_id, client_secret) = self.auth_client()?;
        let response = self
            .client
            .post(TOKEN_URL)
            .header("User-Agent", AUTH_USER_AGENT)
            .form(&[
                ("client_id", client_id.as_str()),
                ("client_secret", client_secret.as_str()),
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ])
            .send()
            .await
            .ok()?;
        let body = json_body(response).await.ok()?;
        let access = super::nav_str(&body, &["access_token"])?;
        let expires_in = body.get("expires_in").and_then(Value::as_u64).unwrap_or(3600);
        self.store_token(&access, None, expires_in);
        Some(access)
    }

    fn auth_client(&self) -> Option<(String, String)> {
        auth_client_of(&self.profile())
    }
}

/// Device client credentials from the profile: `authClientId` / `authClientSecret`.
pub fn auth_client_of(profile: &Profile) -> Option<(String, String)> {
    let id = profile.auth_client_id.clone();
    let secret = profile.auth_client_secret.clone();
    (!id.is_empty() && !secret.is_empty()).then_some((id, secret))
}

#[derive(serde::Serialize, serde::Deserialize)]
struct StoredToken {
    access_token: String,
    refresh_token: Option<String>,
    expires_at: u64,
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

async fn json_body(response: reqwest::Response) -> Result<Value, String> {
    let status = response.status();
    let text = response.text().await.map_err(|e| format!("body read failed: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("not JSON ({status}): {e}"))
}

/// `{device_code, user_code, verification_url, interval, expires_in}`.
pub fn parse_device_code(body: &Value) -> Result<DeviceLogin, String> {
    let field = |key: &str| body.get(key).and_then(Value::as_str).map(str::to_string);
    Ok(DeviceLogin {
        user_code: field("user_code").ok_or("device code missing user_code")?,
        verification_url: field("verification_url")
            .or_else(|| field("verification_uri"))
            .ok_or("device code missing verification_url")?,
        device_code: field("device_code").ok_or("device code missing device_code")?,
        interval_seconds: body.get("interval").and_then(Value::as_u64).unwrap_or(5),
        expires_in_seconds: body.get("expires_in").and_then(Value::as_u64).unwrap_or(1800),
    })
}

/// Token endpoint: granted, still pending, or a definite failure.
pub fn parse_token(status: u16, body: &Value) -> Result<TokenStatus, String> {
    if status == 200 {
        let access_token = body
            .get("access_token")
            .and_then(Value::as_str)
            .ok_or("token response missing access_token")?
            .to_string();
        return Ok(TokenStatus::Granted {
            access_token,
            refresh_token: body.get("refresh_token").and_then(Value::as_str).map(str::to_string),
            expires_in: body.get("expires_in").and_then(Value::as_u64).unwrap_or(3600),
        });
    }
    match body.get("error").and_then(Value::as_str) {
        Some("authorization_pending") | Some("slow_down") => Ok(TokenStatus::Pending),
        Some("access_denied") => Ok(TokenStatus::Denied),
        Some("expired_token") => Ok(TokenStatus::Expired),
        Some(other) => Err(format!("token request rejected: {other}")),
        None => Err(format!("token request failed with status {status}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn device_code_response_maps_both_url_spellings() {
        let login = parse_device_code(&json!({
            "device_code": "d", "user_code": "ABCD-EFGH",
            "verification_url": "https://www.google.com/device",
            "interval": 5, "expires_in": 1800
        }))
        .unwrap();
        assert_eq!(login.user_code, "ABCD-EFGH");
        assert_eq!(login.interval_seconds, 5);

        let uri = parse_device_code(&json!({
            "device_code": "d", "user_code": "U", "verification_uri": "https://x"
        }))
        .unwrap();
        assert_eq!(uri.verification_url, "https://x");
        assert!(parse_device_code(&json!({"user_code": "U"})).is_err());
    }

    #[test]
    fn token_responses_cover_granted_pending_denied_expired() {
        match parse_token(200, &json!({"access_token": "a", "refresh_token": "r", "expires_in": 3600})).unwrap() {
            TokenStatus::Granted { access_token, refresh_token, expires_in } => {
                assert_eq!(access_token, "a");
                assert_eq!(refresh_token.as_deref(), Some("r"));
                assert_eq!(expires_in, 3600);
            }
            other => panic!("expected granted, got {other:?}"),
        }
        assert!(matches!(parse_token(400, &json!({"error": "authorization_pending"})).unwrap(), TokenStatus::Pending));
        assert!(matches!(parse_token(400, &json!({"error": "slow_down"})).unwrap(), TokenStatus::Pending));
        assert!(matches!(parse_token(400, &json!({"error": "access_denied"})).unwrap(), TokenStatus::Denied));
        assert!(matches!(parse_token(400, &json!({"error": "expired_token"})).unwrap(), TokenStatus::Expired));
        assert!(parse_token(400, &json!({"error": "invalid_client"})).is_err());
    }

    #[test]
    fn auth_client_requires_both_halves_from_the_profile() {
        let mut profile = Profile::default();
        assert!(auth_client_of(&profile).is_none());
        profile.auth_client_id = "id".into();
        assert!(auth_client_of(&profile).is_none());
        profile.auth_client_secret = "secret".into();
        assert_eq!(auth_client_of(&profile), Some(("id".into(), "secret".into())));
    }
}
