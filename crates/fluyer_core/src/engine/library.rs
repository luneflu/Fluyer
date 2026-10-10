//! Song list and music folders: scanning, removal, track lookup, playback.

use super::FluyerEngine;
use crate::db::{self, Database};
use crate::library::{self as library_scanner, process_supported_files, scan_directories};
use crate::metadata::MusicMetadata;
use crate::view_models;
use std::path::Path;
use std::sync::Arc;

impl FluyerEngine {
    /// Upserts files under `directories` and drops rows for files under those
    /// same roots that no longer exist. Rows outside the roots are untouched.
    pub fn library_scan(&self, directories: &[String]) {
        let db = Arc::clone(&self.db);
        let library = Arc::clone(&self.library);
        let sink = self.event_sink.clone();
        let dirs = directories.to_vec();

        self.runtime.spawn(async move {
            let file_paths = scan_directories(&dirs);
            // Missing root (unplugged drive) walks as empty: never prune it.
            let roots: Vec<&Path> = dirs.iter().map(Path::new).filter(|p| p.is_dir()).collect();
            let found: std::collections::HashSet<String> =
                file_paths.iter().map(|p| p.display().to_string()).collect();
            prune_rows(&db, |p| !found.contains(p) && roots.iter().any(|r| Path::new(p).starts_with(r)));

            let progress_sink = sink.clone();
            let on_progress = move |cur: usize, tot: usize| {
                if let Some(s) = &progress_sink {
                    s.on_scan_progress(cur, tot);
                }
            };

            let updated_music = process_supported_files(&file_paths, db, Some(on_progress)).await;
            library.write().unwrap().rebuild(updated_music);

            if let Some(s) = &sink {
                s.on_toast("Library scan completed");
            }
        });
    }

    /// Synchronously drops every row under `directory` (removed library folder)
    /// and rebuilds the in-memory library; caller re-reads afterwards.
    pub fn library_remove_folder(&self, directory: &str) {
        let root = Path::new(directory);
        prune_rows(&self.db, |p| Path::new(p).starts_with(root));
        let music = library_scanner::load_all_music_from_db(&self.db);
        self.library.write().unwrap().rebuild(music);
    }

    pub fn library_get_track_count(&self) -> usize {
        self.library.read().unwrap().count()
    }

    pub fn library_get_track(&self, index: usize) -> Option<view_models::TrackItemViewModel> {
        let current_path = self.player.get_current_track().map(|t| t.path);
        self.library.read().unwrap().get_by_index(index).map(|t| {
            let is_current = current_path.as_deref() == Some(&t.path);
            view_models::TrackItemViewModel::from_metadata(index, &t, is_current)
        })
    }

    pub fn library_play_all(&self, start_index: usize) {
        let music = self.library.read().unwrap().music_list.clone();
        if music.is_empty() {
            return;
        }
        self.player.clear();
        self.player.add_track_no_auto_play(music);
        self.player.goto_track(start_index);
    }

    /// Play library tracks in the caller's order (UI sort), starting at `start_index`
    /// of `indices`. Unknown indices are skipped.
    pub fn library_play_tracks(&self, indices: &[usize], start_index: usize) {
        let music: Vec<MusicMetadata> = {
            let library = self.library.read().unwrap();
            indices.iter().filter_map(|&i| library.get_by_index(i)).collect()
        };
        if music.is_empty() {
            return;
        }
        self.player.clear();
        self.player.add_track_no_auto_play(music);
        self.player.goto_track(start_index);
    }
}

/// Deletes DB rows whose path matches `stale`.
fn prune_rows(db: &Database, stale: impl Fn(&str) -> bool) {
    let paths: Vec<String> = db::repo::get_known_music_files(db)
        .unwrap_or_default()
        .into_keys()
        .filter(|p| stale(p))
        .collect();
    if let Err(e) = db::repo::delete_music_paths(db, &paths) {
        crate::flog_err!("Scanner", "Failed to prune {} rows: {}", paths.len(), e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prune_rows_by_folder_is_component_scoped() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        db::migrations::DATABASE_MIGRATIONS.to_latest(&mut conn).unwrap();
        conn.execute_batch(
            "INSERT INTO musics (path, modified_at) VALUES ('/music/a.flac', 't');
             INSERT INTO musics (path, modified_at) VALUES ('/music/sub/b.flac', 't');
             INSERT INTO musics (path, modified_at) VALUES ('/music2/c.flac', 't');",
        )
        .unwrap();
        let db = Database { conn: std::sync::Mutex::new(conn) };

        let root = Path::new("/music");
        prune_rows(&db, |p| Path::new(p).starts_with(root));

        let left: Vec<String> = db::repo::get_known_music_files(&db).unwrap().into_keys().collect();
        assert_eq!(left, vec!["/music2/c.flac".to_string()]);
    }
}
