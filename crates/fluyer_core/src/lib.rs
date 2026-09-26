#[macro_use]
pub mod logger;
pub mod audio;
pub mod db;
pub mod events;
pub mod ffi;
pub mod library;
pub mod metadata;
pub mod services;
pub mod view_models;
pub mod uniffi_api;

uniffi::setup_scaffolding!();

use audio::{MusicPlayer, RepeatMode};
use db::Database;
use events::EventSink;
use library::{LibraryState, process_supported_files, scan_directories};
use metadata::MusicMetadata;
use services::{CoverArtService, DiscordRpc, LyricService};
use std::path::Path;
use std::sync::{Arc, RwLock};

pub struct FluyerEngine {
    pub player: Arc<MusicPlayer>,
    pub db: Arc<Database>,
    pub library: Arc<RwLock<LibraryState>>,
    pub lyrics: Arc<LyricService>,
    pub cover_art: Arc<CoverArtService>,
    pub runtime: tokio::runtime::Runtime,
    pub event_sink: Option<Arc<dyn EventSink>>,
}

impl FluyerEngine {
    pub fn new(
        data_dir: &Path,
        cache_dir: &Path,
        event_sink: Option<Arc<dyn EventSink>>,
    ) -> Result<Self, String> {
        let _ = std::fs::create_dir_all(data_dir);
        let _ = std::fs::create_dir_all(cache_dir);

        let db_path = data_dir.join(db::DATABASE_NAME);
        let database = Arc::new(Database::open(&db_path)?);

        let lyrics = Arc::new(LyricService::new(cache_dir));
        let cover_art = Arc::new(CoverArtService::new(cache_dir));

        let player = Arc::new(MusicPlayer::new(event_sink.clone()));
        let library = Arc::new(RwLock::new(LibraryState::default()));

        // Preload library from database
        let initial_music = library::load_all_music_from_db(&database);
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
        })
    }

    pub fn scan_and_update(&self, directories: &[String]) {
        let db = Arc::clone(&self.db);
        let library = Arc::clone(&self.library);
        let sink = self.event_sink.clone();
        let dirs = directories.to_vec();

        self.runtime.spawn(async move {
            let file_paths = scan_directories(&dirs);
            let progress_sink = sink.clone();
            let on_progress = move |cur: usize, tot: usize| {
                if let Some(ref s) = progress_sink {
                    s.on_scan_progress(cur, tot);
                }
            };

            let updated_music = process_supported_files(&file_paths, db, Some(on_progress)).await;
            library.write().unwrap().rebuild(updated_music);

            if let Some(ref s) = sink {
                s.on_toast("Library scan completed");
            }
        });
    }

    pub fn play(&self) {
        self.player.play();
    }

    pub fn pause(&self) {
        self.player.pause();
    }

    pub fn toggle_play(&self) {
        self.player.toggle_play();
    }

    pub fn next(&self) {
        self.player.next();
    }

    pub fn previous(&self) {
        self.player.previous();
    }

    pub fn seek(&self, position_ms: u64) {
        self.player.set_pos(position_ms);
    }

    pub fn set_volume(&self, volume: f32) {
        self.player.set_volume(volume);
    }

    pub fn set_repeat_mode(&self, mode: RepeatMode) {
        self.player.set_repeat_mode(mode);
    }

    pub fn shuffle(&self) {
        self.player.shuffle_track();
    }

    pub fn play_single_from_library(&self, index: usize) {
        if let Some(track) = self.library.read().unwrap().get_by_index(index) {
            self.player.clear();
            self.player.add_track(vec![track.clone()]);
        }
    }

    pub fn play_all_from_library(&self, start_index: usize) {
        let music = self.library.read().unwrap().music_list.clone();
        if music.is_empty() {
            return;
        }
        self.player.clear();
        self.player.add_track_no_auto_play(music);
        self.player.goto_track(start_index);
    }

    pub fn play_album(&self, index: usize) {
        if let Some(album) = self.library.read().unwrap().album_get_by_index(index) {
            if album.is_empty() {
                return;
            }
            self.player.clear();
            self.player.add_track_no_auto_play(album);
            self.player.goto_track(0);
        }
    }

    pub fn play_album_track(&self, album_index: usize, track_index: usize) {
        if let Some(album) = self.library.read().unwrap().album_get_by_index(album_index) {
            if track_index < album.len() {
                self.player.clear();
                self.player.add_track_no_auto_play(album);
                self.player.goto_track(track_index);
            }
        }
    }

    pub fn queue_album(&self, index: usize) {
        if let Some(album) = self.library.read().unwrap().album_get_by_index(index) {
            if album.is_empty() {
                return;
            }
            self.player.add_track(album);
        }
    }

    pub fn shuffle_album(&self, index: usize) {
        if let Some(album) = self.library.read().unwrap().album_get_by_index(index) {
            if album.is_empty() {
                return;
            }
            self.player.clear();
            self.player.add_track_no_auto_play(album);
            self.player.shuffle_track();
            self.player.goto_track(0);
        }
    }

    pub fn add_track_to_queue(&self, track: MusicMetadata) {
        self.player.add_track(vec![track]);
    }

    pub fn resolve_track_cover(&self, track: &MusicMetadata, notify_index: Option<usize>) -> Option<Vec<u8>> {
        let artist = track.artist.as_deref().unwrap_or("");
        let album = track.album.as_deref();
        let title = track.title.as_deref();

        // 1. Check disk cache first (fast, hits SSD/OS cache, 0 HDD reads)
        if let Some(cached) = self.cover_art.get_cached_with_fallback(artist, album, title, Some(&track.path)) {
            return Some(cached);
        }

        // 2. Extract embedded cover art with Lofty (only parses header, avoids decoding audio)
        if let Ok(bytes) = MusicMetadata::get_image_with_lofty(&track.path) {
            let _ = self.cover_art.save_to_cache(artist, album, title, Some(&track.path), &bytes);
            return Some(bytes);
        }

        // 3. Fallback: async fetch online from MusicBrainz / Cover Art Archive
        let cover_service = Arc::clone(&self.cover_art);
        let sink = self.event_sink.clone();
        let artist_s = artist.to_string();
        let album_s = album.map(|s| s.to_string());
        let title_s = title.map(|s| s.to_string());
        self.runtime.spawn(async move {
            if let Ok(Some(_)) = cover_service.fetch_and_cache(&artist_s, album_s.as_deref(), title_s.as_deref()).await {
                if let (Some(s), Some(idx)) = (sink, notify_index) {
                    s.on_track_cover_loaded(idx);
                }
            }
        });
        None
    }

    pub fn resolve_album_cover(&self, album: &[MusicMetadata], notify_index: Option<usize>) -> Option<Vec<u8>> {
        let first_track = album.first()?;
        let artist = first_track.album_artist.as_deref()
            .or(first_track.artist.as_deref())
            .unwrap_or("");
        let album_name = first_track.album.as_deref();

        // 1. Check disk cache first
        if let Some(cached) = self.cover_art.get_cached_with_fallback(artist, album_name, None, Some(&first_track.path)) {
            return Some(cached);
        }

        // 2. Extract embedded cover art from first track with Lofty
        if let Ok(bytes) = MusicMetadata::get_image_with_lofty(&first_track.path) {
            let _ = self.cover_art.save_to_cache(artist, album_name, None, Some(&first_track.path), &bytes);
            return Some(bytes);
        }

        // 3. Fallback: async fetch online
        let cover_service = Arc::clone(&self.cover_art);
        let sink = self.event_sink.clone();
        let artist_s = artist.to_string();
        let album_s = album_name.map(|s| s.to_string());
        self.runtime.spawn(async move {
            if let Ok(Some(_)) = cover_service.fetch_and_cache(&artist_s, album_s.as_deref(), None).await {
                if let (Some(s), Some(idx)) = (sink, notify_index) {
                    s.on_album_cover_loaded(idx);
                }
            }
        });
        None
    }

    pub fn generate_background_for_current(&self, width: u32, height: u32) -> (Vec<u8>, u32, u32) {
        let cover_bytes = self
            .player
            .get_current_track()
            .and_then(|t| self.resolve_track_cover(&t, None));
        let colors = if let Some(bytes) = cover_bytes {
            services::background::extract_prominent_from_bytes(&bytes, 10, false)
        } else {
            vec![services::background::balance_color([30, 30, 40], true)]
        };
        let blurred = services::background::generate_blurred_background(&colors, width, height);
        let (w, h) = (blurred.width(), blurred.height());
        (blurred.into_raw(), w, h)
    }

    pub fn resolve_lyrics(&self, track: &MusicMetadata) -> Option<String> {
        if let Some(lyrics) = metadata::probe::extract_lyrics_file(&track.path) {
            return Some(lyrics);
        }
        if let Some(lyrics) = metadata::probe::extract_lyrics_embedded(&track.path) {
            return Some(lyrics);
        }
        let title = track.title.as_deref().unwrap_or("");
        let artist = track.artist.as_deref().unwrap_or("");
        let duration = track.duration.map(|d| d as u64);
        let query = services::lyric::LyricQuery {
            title: title.to_string(),
            artist: artist.to_string(),
            duration,
        };
        if let Some(cached) = self.lyrics.get_cached(&query) {
            return Some(cached);
        }
        let lyric_service = Arc::clone(&self.lyrics);
        let sink = self.event_sink.clone();
        self.runtime.spawn(async move {
            if let Some(fetched) = lyric_service.fetch(query).await {
                if let Some(s) = sink {
                    s.on_lyrics_loaded(&fetched);
                }
            }
        });
        None
    }

    pub fn get_track_view(&self, index: usize) -> Option<view_models::TrackItemViewModel> {
        let current_path = self.player.get_current_track().map(|t| t.path);
        self.library
            .read()
            .unwrap()
            .get_by_index(index)
            .map(|t| {
                let is_current = current_path.as_deref() == Some(&t.path);
                view_models::TrackItemViewModel::from_metadata(index, &t, is_current)
            })
    }

    pub fn get_album_card(&self, index: usize) -> Option<view_models::AlbumCardViewModel> {
        self.library
            .read()
            .unwrap()
            .album_get_by_index(index)
            .map(|tracks| view_models::AlbumCardViewModel::from_tracks(index, &tracks))
    }

    pub fn get_album_detail(&self, index: usize) -> Option<view_models::AlbumDetailViewModel> {
        let current_path = self.player.get_current_track().map(|t| t.path);
        self.library
            .read()
            .unwrap()
            .album_get_by_index(index)
            .map(|tracks| {
                view_models::AlbumDetailViewModel::from_tracks(
                    index,
                    &tracks,
                    current_path.as_deref(),
                )
            })
    }

    pub fn get_album_view(&self, index: usize) -> Option<view_models::AlbumCardViewModel> {
        self.get_album_card(index)
    }

    pub fn get_player_bar_view(&self) -> view_models::PlayerBarViewModel {
        let sync = self.player.get_sync_info(false);
        let current_track = self.player.get_current_track();
        let (title, artist, album) = match current_track {
            Some(ref t) => (
                t.title.clone().unwrap_or_else(|| metadata::DEFAULT_TITLE.to_string()),
                t.artist.clone().unwrap_or_else(|| metadata::DEFAULT_ARTIST.to_string()),
                t.album.clone().unwrap_or_default(),
            ),
            None => ("No Track".to_string(), String::new(), String::new()),
        };

        let position_ms = sync.position_ms();
        let duration_ms = sync.duration_ms();
        let progress_pct = if duration_ms > 0 {
            (position_ms as f32 / duration_ms as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let time_label = format!(
            "{} / {}",
            view_models::format_time(position_ms),
            view_models::format_time(duration_ms)
        );

        view_models::PlayerBarViewModel {
            track_index: sync.index,
            title,
            artist,
            album,
            position_ms,
            duration_ms,
            progress_pct,
            time_label,
            is_playing: sync.is_playing,
            repeat_mode: sync.repeat_mode.into(),
            is_shuffled: sync.is_shuffled,
            volume: 1.0,
        }
    }

    pub fn get_play_view(&self) -> view_models::PlayViewModel {
        let current_track = self.player.get_current_track();
        let track_vm = current_track.as_ref().map(|t| {
            let sync = self.player.get_sync_info(false);
            let idx = if sync.index >= 0 { sync.index as usize } else { 0 };
            view_models::TrackItemViewModel::from_metadata(idx, t, true)
        });

        let lyrics = self.get_parsed_lyrics();
        let pos_ms = self.player.get_sync_info(false).position_ms();
        let current_lyric_index = view_models::find_active_lyric_index(&lyrics, pos_ms);

        let cover_bytes = current_track
            .as_ref()
            .and_then(|t| self.resolve_track_cover(t, None));
        let colors = if let Some(bytes) = cover_bytes {
            services::background::extract_prominent_from_bytes(&bytes, 5, false)
        } else {
            vec![services::background::balance_color([30, 30, 40], true)]
        };
        let palette = colors
            .into_iter()
            .map(|c| view_models::ColorRgb {
                r: c[0],
                g: c[1],
                b: c[2],
            })
            .collect();

        view_models::PlayViewModel {
            track: track_vm,
            lyrics,
            current_lyric_index,
            palette,
        }
    }

    pub fn get_parsed_lyrics(&self) -> Vec<view_models::LyricLine> {
        let lyrics_text = self
            .player
            .get_current_track()
            .and_then(|t| self.resolve_lyrics(&t));
        match lyrics_text {
            Some(ref lrc) => view_models::parse_lrc(lrc),
            None => Vec::new(),
        }
    }
}
