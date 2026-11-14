//! Login values crossing to the UI. The device code itself is not secret (it is
//! useless without the grant), but the tokens never leave the core's secret store.

#[derive(uniffi::Record, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeviceLogin {
    /// Short code the user types on the verification page.
    pub user_code: String,
    pub verification_url: String,
    pub device_code: String,
    pub interval_seconds: u64,
    pub expires_in_seconds: u64,
}

#[derive(uniffi::Enum, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TokenStatus {
    /// Authorisation accepted; stored in the secret store already.
    Granted { access_token: String, refresh_token: Option<String>, expires_in: u64 },
    /// User has not approved yet; poll again after `interval_seconds`.
    Pending,
    Denied,
    Expired,
}
