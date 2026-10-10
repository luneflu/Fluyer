/// Process-global in the core (not per engine).
#[no_mangle]
pub unsafe extern "C" fn fluyer_discord_set_enabled(enabled: bool) {
    crate::services::DiscordRpc::set_enabled(enabled);
}
