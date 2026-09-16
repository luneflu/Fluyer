use crate::db::repo::{self, MusicRecord};
use crate::db::Database;
use crate::metadata::MusicMetadata;
use chrono::{DateTime, Utc};
use futures::StreamExt;
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
                .filter_entry(is_not_hidden)
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
    let existing_records = Arc::new(repo::get_known_music_files(&db).unwrap_or_default());
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
        let mut records = Vec::with_capacity(metadata_results.len());
        for (idx, (path, modified_at, metadata)) in metadata_results.into_iter().enumerate() {
            if let Some(ref cb) = on_progress {
                cb(idx + 1, total);
            }

            if let Ok(meta) = metadata {
                records.push(MusicRecord {
                    path,
                    modified_at: modified_at.unwrap_or_default(),
                    metadata: meta,
                });
            }
        }
        let _ = repo::upsert_music_batch(&db, &records);
    }

    repo::load_all_music(&db)
}

fn get_modified_time(path: &Path) -> Option<String> {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|mtime| DateTime::<Utc>::from(mtime).to_rfc3339())
}
