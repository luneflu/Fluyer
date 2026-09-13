use crate::db::Database;
use crate::metadata::MusicMetadata;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::{DirEntry, WalkDir};

const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "wav", "ogg", "m4a", "m4b", "mp4", "mkv", "webm", "avi", "aac", "wma", "opus",
    "alac", "ape", "aiff", "mov", "ts", "flv", "3gp", "wmv",
];

fn is_not_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| !s.starts_with('.'))
        .unwrap_or(false)
}

pub fn scan_directories(search_dirs: &[String]) -> Vec<PathBuf> {
    let mut dirs: Vec<Result<DirEntry, walkdir::Error>> = vec![];

    for dir in search_dirs {
        dirs.extend(
            WalkDir::new(dir)
                .into_iter()
                .filter_entry(|e| is_not_hidden(e))
                .collect::<Vec<_>>(),
        );
    }

    dirs.into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            if !e.path().is_file() {
                return false;
            }

            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .map(|entry| entry.path().to_path_buf())
        .collect()
}

pub async fn process_supported_files<F>(
    paths: &[PathBuf],
    db: Arc<Database>,
    on_progress: Option<F>,
) -> Vec<MusicMetadata>
where
    F: Fn(usize, usize) + Send + Sync + 'static,
{
    let existing_records = {
        let conn = db.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT path, modified_at FROM musics")
            .unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
            })
            .unwrap();

        let mut map = HashMap::new();
        for row in rows.flatten() {
            map.insert(row.0, row.1.unwrap_or_default());
        }
        map
    };

    let existing_records = Arc::new(existing_records);
    let total = paths.len();

    let metadata_results: Vec<_> = futures::stream::iter(paths.to_vec())
        .map(|path| {
            let existing_records = existing_records.clone();
            async move {
                let path_str = path.display().to_string();
                let modified = get_modified_time(&path);

                if let Some(db_modified) = existing_records.get(&path_str) {
                    if let Some(curr_modified) = &modified {
                        if db_modified == curr_modified {
                            return None;
                        }
                    }
                }

                let metadata = MusicMetadata::get(path_str.clone()).await;
                Some((path_str, modified, metadata))
            }
        })
        .buffer_unordered(10)
        .filter_map(|res| async { res })
        .collect()
        .await;

    if !metadata_results.is_empty() {
        let mut conn = db.conn.lock().unwrap();
        let tx = conn.transaction().unwrap();

        for (idx, (path, modified_at, metadata)) in metadata_results.into_iter().enumerate() {
            if let Some(ref cb) = on_progress {
                cb(idx + 1, total);
            }

            if let Ok(metadata) = metadata {
                let path_string = path.to_string();
                let modified_at = modified_at.unwrap_or_default();

                let _ = tx.execute(
                    "INSERT INTO musics (
                        path, duration, title, artist, album, album_artist,
                        track_number, genre, date, bits_per_sample, sample_rate, modified_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                    ON CONFLICT(path) DO UPDATE SET
                        duration = excluded.duration,
                        title = excluded.title,
                        artist = excluded.artist,
                        album = excluded.album,
                        album_artist = excluded.album_artist,
                        track_number = excluded.track_number,
                        genre = excluded.genre,
                        date = excluded.date,
                        bits_per_sample = excluded.bits_per_sample,
                        sample_rate = excluded.sample_rate,
                        modified_at = excluded.modified_at",
                    rusqlite::params![
                        path_string,
                        metadata.duration.map(|d| d as i64),
                        metadata.title,
                        metadata.artist,
                        metadata.album,
                        metadata.album_artist,
                        metadata.track_number,
                        metadata.genre,
                        metadata.date,
                        metadata.bits_per_sample.map(|b| b as i64),
                        metadata.sample_rate.map(|s| s as i64),
                        modified_at
                    ],
                );
            }
        }
        let _ = tx.commit();
    }

    load_all_music_from_db(&db)
}

pub fn load_all_music_from_db(db: &Database) -> Vec<MusicMetadata> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT id, path, duration, title, artist, album, album_artist,
                    track_number, genre, date, bits_per_sample, sample_rate
             FROM musics ORDER BY id ASC",
        );

    let mut stmt = match stmt {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to prepare SQL query for musics: {}", e);
            return Vec::new();
        }
    };

    let rows = stmt.query_map([], |row| {
        let path_str: String = row.get(1)?;
        let filename = Path::new(&path_str)
            .file_name()
            .and_then(|f| f.to_str())
            .map(|s| s.to_string());

        Ok(MusicMetadata {
            id: row.get(0)?,
            path: path_str,
            duration: row.get::<_, Option<i64>>(2)?.map(|d| d as u128),
            filename,
            title: row.get(3)?,
            artist: row.get(4)?,
            album: row.get(5)?,
            album_artist: row.get(6)?,
            track_number: row.get(7)?,
            genre: row.get(8)?,
            date: row.get(9)?,
            bits_per_sample: row.get::<_, Option<i64>>(10)?.map(|b| b as u32),
            sample_rate: row.get::<_, Option<i64>>(11)?.map(|s| s as u32),
            image: None,
            extra_tags: None,
        })
    });

    match rows {
        Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
        Err(e) => {
            log::error!("Query map failed: {}", e);
            Vec::new()
        }
    }
}

fn get_modified_time(path: &Path) -> Option<String> {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|mtime| DateTime::<Utc>::from(mtime).to_rfc3339())
}
