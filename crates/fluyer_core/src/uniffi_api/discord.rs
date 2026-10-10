use super::FluyerAppEngine;

#[uniffi::export]
impl FluyerAppEngine {
    /// Process-global in the core (not per engine).
    pub fn discord_set_enabled(&self, enabled: bool) {
        crate::services::DiscordRpc::set_enabled(enabled);
    }
}
