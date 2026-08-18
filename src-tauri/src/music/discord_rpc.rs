use discord_rich_presence::{
    activity::{Activity, ActivityType, Assets, Timestamps},
    DiscordIpc, DiscordIpcClient,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DISCORD_APP_ID_ENV: &str = "DISCORD_APPLICATION_ID";
const LARGE_IMAGE_KEY: &str = "fluyer_logo";
const RECONNECT_INTERVAL: Duration = Duration::from_secs(30);

static ENABLED: AtomicBool = AtomicBool::new(false);
static CLIENT: OnceLock<DiscordRpc> = OnceLock::new();

pub struct DiscordRpc {
    tx: mpsc::Sender<Message>,
}

pub struct ActivityData {
    pub title: String,
    pub artist: Option<String>,
    pub position_ms: Option<f64>,
    pub duration_ms: Option<u128>,
    pub is_playing: bool,
}

enum Message {
    Set(ActivityData),
    Clear,
    Shutdown,
}

fn client_id() -> Option<String> {
    match std::env::var(DISCORD_APP_ID_ENV) {
        Ok(id) if !id.trim().is_empty() => Some(id),
        _ => None,
    }
}

impl DiscordRpc {
    pub fn set_enabled(enabled: bool) {
        ENABLED.store(enabled, Ordering::SeqCst);
        if enabled {
            let _ = Self::instance();
        } else {
            Self::clear();
        }
    }

    fn instance() -> Option<&'static DiscordRpc> {
        if !ENABLED.load(Ordering::SeqCst) {
            return None;
        }
        Some(CLIENT.get_or_init(Self::start))
    }

    pub fn update(data: ActivityData) {
        if let Some(rpc) = Self::instance() {
            let _ = rpc.tx.send(Message::Set(data));
        }
    }

    pub fn clear() {
        if let Some(rpc) = CLIENT.get() {
            let _ = rpc.tx.send(Message::Clear);
        }
    }

    pub fn shutdown() {
        if let Some(rpc) = CLIENT.get() {
            let _ = rpc.tx.send(Message::Shutdown);
        }
    }

    fn start() -> DiscordRpc {
        let (tx, rx) = mpsc::channel();

        std::thread::Builder::new()
            .name("discord-rpc".to_string())
            .spawn(move || {
                if client_id().is_none() {
                    crate::debug!("Discord Rich Presence disabled: {} not set", DISCORD_APP_ID_ENV);
                    return;
                }

                let mut client = DiscordIpcClient::new(client_id().unwrap());
                let mut connected = false;
                let mut last: Option<ActivityData> = None;

                loop {
                    match rx.recv_timeout(RECONNECT_INTERVAL) {
                        Ok(Message::Set(data)) => {
                            if !connected && client.connect().is_ok() {
                                connected = true;
                            }
                            if connected && client.set_activity(build_activity(&data)).is_err() {
                                crate::warn!("Discord RPC: failed to set activity, reconnecting");
                                let _ = client.close();
                                connected = false;
                            }
                            last = Some(data);
                        }
                        Ok(Message::Clear) => {
                            if connected && client.clear_activity().is_err() {
                                crate::warn!("Discord RPC: failed to clear activity");
                            }
                            last = None;
                        }
                        Ok(Message::Shutdown) => {
                            if connected {
                                let _ = client.clear_activity();
                                let _ = client.close();
                            }
                            break;
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            // Retry connecting so presence shows up if Discord starts later
                            if !connected && client.connect().is_ok() {
                                connected = true;
                                crate::info!("Discord RPC: connected");
                                if let Some(data) = last.as_ref() {
                                    let _ = client.set_activity(build_activity(data));
                                }
                            }
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                }
            })
            .expect("failed to spawn Discord RPC thread");

        DiscordRpc { tx }
    }
}

fn build_activity(data: &ActivityData) -> Activity<'static> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let mut activity = Activity::new()
        .activity_type(ActivityType::Listening)
        .details(data.title.clone());

    if data.is_playing {
        activity = activity.state(data.artist.clone().unwrap_or_else(|| "Unknown Artist".to_string()));

        let pos = data.position_ms.unwrap_or(0.0).max(0.0) as i64;
        let start = now - pos;
        let mut timestamps = Timestamps::new().start(start);
        if let Some(duration) = data.duration_ms {
            timestamps = timestamps.end(start + duration as i64);
        }
        activity = activity.timestamps(timestamps);
    } else {
        activity = activity.state("Paused".to_string());
    }

    activity.assets(Assets::new().large_image(LARGE_IMAGE_KEY))
}