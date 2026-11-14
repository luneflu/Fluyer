//! `FluyerEngine`: the core every UI drives. One file per feature, same names as
//! the FFI layers (`uniffi_api/`, `c_api/`): `queue_get` lives in `queue.rs`.

mod album;
mod artwork;
mod backdrop;
mod library;
mod lyrics;
mod player;
mod provider;
mod queue;
mod settings;
mod thumbnail_cache;

use crate::audio::MusicPlayer;
use crate::db::{self, Database};
use crate::events::EventSink;
use crate::library::LibraryState;
use crate::services::ytmusic::YouTubeMusic;
use crate::services::{
    CoverArtService, DiscordRpc, LyricService, ProfileService, Secrets, SettingsService, StreamProxy,
};
use crate::{logger, view_models};
use std::path::Path;
use std::sync::{Arc, RwLock};
use thumbnail_cache::ThumbnailCache;

pub struct FluyerEngine {
    pub player: Arc<MusicPlayer>,
    pub db: Arc<Database>,
    pub library: Arc<RwLock<LibraryState>>,
    pub lyrics: Arc<LyricService>,
    pub cover_art: Arc<CoverArtService>,
    pub settings: Arc<SettingsService>,
    pub secrets: Arc<Secrets>,
    pub profile: Arc<ProfileService>,
    /// Loopback HTTP bridge for provider streams (BASS has no TLS).
    pub stream_proxy: Arc<StreamProxy>,
    pub data_dir: std::path::PathBuf,
    pub cache_dir: std::path::PathBuf,
    // ponytail: one provider instance per applied profile version; rebuilt lazily so a
    // confirmed profile update takes effect without a restart.
    provider_cache: RwLock<Option<(u32, Arc<YouTubeMusic>)>>,
    pub runtime: tokio::runtime::Runtime,
    pub event_sink: Option<Arc<dyn EventSink>>,
    // ponytail: caches parsed LRC per track path; the UI polls this every 250ms
    // and re-reading + re-parsing the .lrc file each tick stalled the main thread.
    lyrics_cache: Arc<RwLock<Option<(String, Arc<Vec<view_models::LyricLine>>)>>>,
    // ponytail: memoized down-scaled JPEG covers. Decoding a 3000x3000 cover is
    // O(pixels), so scroll views would otherwise re-decode on every cell render.
    thumbnail_cache: Arc<RwLock<ThumbnailCache>>,
    palette_cache: Arc<RwLock<Option<(String, Arc<Vec<[u8; 3]>>)>>>,
}

impl FluyerEngine {
    /// Runs `future` on the engine's tokio runtime and awaits it from whatever executor
    /// the caller has. UniFFI polls exported `async` methods on its own executor, so
    /// anything touching `reqwest` must come through here ("no reactor running").
    pub async fn on_runtime<F, T>(&self, future: F) -> T
    where
        F: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.runtime.spawn(async move {
            let _ = tx.send(future.await);
        });
        rx.await.expect("engine runtime dropped")
    }

    pub fn new(
        data_dir: &Path,
        cache_dir: &Path,
        event_sink: Option<Arc<dyn EventSink>>,
    ) -> Result<Self, String> {
        logger::init();
        let _ = std::fs::create_dir_all(data_dir);
        let _ = std::fs::create_dir_all(cache_dir);

        let db_path = data_dir.join(db::DATABASE_NAME);
        let database = Arc::new(Database::open(&db_path)?);

        let lyrics = Arc::new(LyricService::new(cache_dir));
        let cover_art = Arc::new(CoverArtService::new(cache_dir));
        let settings = Arc::new(SettingsService::new(data_dir));
        let secrets = Arc::new(Secrets::new(data_dir));
        // ponytail: no release URL is baked in yet; a dev can point checks at a file URL.
        let profile_source = std::env::var("FLUYER_PROFILE_URL").ok();
        let profile = Arc::new(ProfileService::new(data_dir, profile_source));

        let player = Arc::new(MusicPlayer::new(event_sink.clone()));
        let library = Arc::new(RwLock::new(LibraryState::default()));

        // Preload library from database
        let initial_music = crate::library::load_all_music_from_db(&database);
        library.write().unwrap().rebuild(initial_music);

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Failed to create async runtime: {}", e))?;

        let stream_proxy = Arc::new(StreamProxy::new());
        stream_proxy.spawn(runtime.handle())?;

        DiscordRpc::init();

        Ok(Self {
            player,
            db: database,
            library,
            lyrics,
            cover_art,
            settings,
            secrets,
            profile,
            stream_proxy,
            data_dir: data_dir.to_path_buf(),
            cache_dir: cache_dir.to_path_buf(),
            provider_cache: RwLock::new(None),
            runtime,
            event_sink,
            lyrics_cache: Arc::new(RwLock::new(None)),
            thumbnail_cache: Arc::new(RwLock::new(ThumbnailCache::new())),
            palette_cache: Arc::new(RwLock::new(None)),
        })
    }
}
