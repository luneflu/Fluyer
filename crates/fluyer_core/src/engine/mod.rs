//! `FluyerEngine`: the core every UI drives. One file per feature, same names as
//! the FFI layers (`uniffi_api/`, `c_api/`): `queue_get` lives in `queue.rs`.

mod album;
mod artwork;
mod backdrop;
mod library;
mod lyrics;
mod player;
mod queue;
mod thumbnail_cache;

use crate::audio::MusicPlayer;
use crate::db::{self, Database};
use crate::events::EventSink;
use crate::library::LibraryState;
use crate::services::{CoverArtService, DiscordRpc, LyricService};
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

        let player = Arc::new(MusicPlayer::new(event_sink.clone()));
        let library = Arc::new(RwLock::new(LibraryState::default()));

        // Preload library from database
        let initial_music = crate::library::load_all_music_from_db(&database);
        library.write().unwrap().rebuild(initial_music);

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Failed to create async runtime: {}", e))?;

        DiscordRpc::init();

        Ok(Self {
            player,
            db: database,
            library,
            lyrics,
            cover_art,
            runtime,
            event_sink,
            lyrics_cache: Arc::new(RwLock::new(None)),
            thumbnail_cache: Arc::new(RwLock::new(ThumbnailCache::new())),
            palette_cache: Arc::new(RwLock::new(None)),
        })
    }
}
