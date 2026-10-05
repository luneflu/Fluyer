use super::Database;
use crate::metadata::MusicMetadata;
use std::collections::HashMap;
use std::path::Path;

pub struct MusicRecord {
    pub path: String,
    pub modified_at: String,
    pub metadata: MusicMetadata,
}

pub fn get_known_music_files(db: &Database) -> Result<HashMap<String, String>, String> {
    let conn = db
        .conn
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare("SELECT path, modified_at FROM musics")
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
        })
        .map_err(|e| format!("Failed to query known files: {}", e))?;

    let mut map = HashMap::new();
    for row in rows.flatten() {
        map.insert(row.0, row.1.unwrap_or_default());
    }
    Ok(map)
}

/// Writes one row per record, keyed on `path`.
///
/// ponytail: this used to be a single `INSERT ... ON CONFLICT(path) DO UPDATE`, which
/// SQLite rejects outright because `musics.path` carries no UNIQUE constraint — so every
/// scan failed to persist anything, and the error was discarded (`let _ =`) so the scan
/// still reported success with an empty library.
///
/// Update-then-insert needs no unique index, so it works against the schema as it
/// already exists rather than requiring a migration, and it keeps `id` stable for rows
/// that are already present.
pub fn upsert_music_batch(db: &Database, records: &[MusicRecord]) -> Result<(), String> {
    if records.is_empty() {
        return Ok(());
    }

    let mut conn = db
        .conn
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;

    let tx = conn
        .transaction()
        .map_err(|e| format!("Transaction begin failed: {}", e))?;

    // Keep these two column lists in step: the insert names every column, the update
    // covers every column except `path`.
    let insert_sql = "INSERT INTO musics (
            path, duration, title, artist, album, album_artist,
            track_number, genre, date, bits_per_sample, sample_rate, modified_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)";
    let update_sql = "UPDATE musics SET
            duration = ?2, title = ?3, artist = ?4, album = ?5, album_artist = ?6,
            track_number = ?7, genre = ?8, date = ?9, bits_per_sample = ?10,
            sample_rate = ?11, modified_at = ?12
        WHERE path = ?1";

    for record in records {
        let params = rusqlite::params![
            record.path,
            record.metadata.duration.map(|d| d as i64),
            record.metadata.title,
            record.metadata.artist,
            record.metadata.album,
            record.metadata.album_artist,
            record.metadata.track_number,
            record.metadata.genre,
            record.metadata.date,
            record.metadata.bits_per_sample.map(|b| b as i64),
            record.metadata.sample_rate.map(|s| s as i64),
            record.modified_at,
        ];

        let updated = tx
            .execute(update_sql, params)
            .map_err(|e| format!("Failed to update '{}': {}", record.path, e))?;

        if updated == 0 {
            tx.execute(insert_sql, params)
                .map_err(|e| format!("Failed to insert '{}': {}", record.path, e))?;
        }
    }

    tx.commit()
        .map_err(|e| format!("Transaction commit failed: {}", e))?;
    Ok(())
}

/// Deletes the rows for `paths`. Returns rows removed.
pub fn delete_music_paths(db: &Database, paths: &[String]) -> Result<usize, String> {
    if paths.is_empty() {
        return Ok(0);
    }
    let mut conn = db
        .conn
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let tx = conn
        .transaction()
        .map_err(|e| format!("Transaction begin failed: {}", e))?;
    let mut removed = 0;
    {
        let mut stmt = tx
            .prepare("DELETE FROM musics WHERE path = ?1")
            .map_err(|e| format!("Failed to prepare delete: {}", e))?;
        for path in paths {
            removed += stmt
                .execute([path])
                .map_err(|e| format!("Failed to delete '{}': {}", path, e))?;
        }
    }
    tx.commit()
        .map_err(|e| format!("Transaction commit failed: {}", e))?;
    Ok(removed)
}

pub fn load_all_music(db: &Database) -> Vec<MusicMetadata> {
    let conn = match db.conn.lock() {
        Ok(c) => c,
        Err(e) => {
            log::error!("DB lock error: {}", e);
            return Vec::new();
        }
    };

    let stmt = conn.prepare(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::DATABASE_MIGRATIONS;
    use std::sync::Mutex;

    fn db() -> Database {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        DATABASE_MIGRATIONS.to_latest(&mut conn).unwrap();
        Database {
            conn: Mutex::new(conn),
        }
    }

    fn record(path: &str, title: &str, modified_at: &str) -> MusicRecord {
        MusicRecord {
            path: path.to_string(),
            modified_at: modified_at.to_string(),
            metadata: MusicMetadata {
                id: 0,
                path: path.to_string(),
                filename: None,
                duration: Some(180_000),
                title: Some(title.to_string()),
                artist: Some("Artist".into()),
                album: Some("Album".into()),
                album_artist: None,
                track_number: Some("3".into()),
                genre: None,
                date: None,
                bits_per_sample: Some(16),
                sample_rate: Some(44_100),
                image: None,
                extra_tags: None,
            },
        }
    }

    fn rows(db: &Database) -> Vec<(i64, String, String, String)> {
        let conn = db.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, path, title, modified_at FROM musics ORDER BY id")
            .unwrap();
        let collected = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        collected
    }

    /// Regression: the upsert used `ON CONFLICT(path)`, which SQLite rejects because
    /// `musics.path` has no UNIQUE constraint — so scanning persisted nothing.
    #[test]
    fn inserts_new_rows() {
        let db = db();
        upsert_music_batch(&db, &[record("/m/a.flac", "A", "t0")]).unwrap();

        assert_eq!(
            rows(&db),
            vec![(1, "/m/a.flac".into(), "A".into(), "t0".into())]
        );
    }

    /// A rescan must update in place rather than adding a second row for the same path,
    /// and must keep `id` stable.
    #[test]
    fn updates_existing_rows_without_duplicating() {
        let db = db();
        upsert_music_batch(&db, &[record("/m/a.flac", "A", "t0")]).unwrap();
        upsert_music_batch(&db, &[record("/m/a.flac", "A (Remastered)", "t1")]).unwrap();

        assert_eq!(
            rows(&db),
            vec![(1, "/m/a.flac".into(), "A (Remastered)".into(), "t1".into())]
        );
    }

    #[test]
    fn mixed_batch_handles_both_paths() {
        let db = db();
        upsert_music_batch(&db, &[record("/m/a.flac", "A", "t0")]).unwrap();
        upsert_music_batch(
            &db,
            &[
                record("/m/a.flac", "A2", "t1"),
                record("/m/b.flac", "B", "t0"),
            ],
        )
        .unwrap();

        let rows = rows(&db);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], (1, "/m/a.flac".into(), "A2".into(), "t1".into()));
        assert_eq!(rows[1].0, 2);
    }

    #[test]
    fn empty_batch_is_a_no_op() {
        let db = db();
        upsert_music_batch(&db, &[]).unwrap();
        assert!(rows(&db).is_empty());
    }

    /// Some databases in the wild already carry a unique index on `path`. The upsert
    /// must work there too, and must never reach the INSERT half for a path that exists.
    #[test]
    fn works_when_a_unique_index_on_path_already_exists() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        DATABASE_MIGRATIONS.to_latest(&mut conn).unwrap();
        conn.execute_batch(
            "CREATE UNIQUE INDEX idx_musics_path ON musics (path);
             INSERT INTO musics (path, title, modified_at) VALUES ('/m/a.flac', 'stale', 't-old');",
        )
        .unwrap();
        let db = Database {
            conn: Mutex::new(conn),
        };

        // Rescan of the existing row, plus a brand new one in the same batch.
        upsert_music_batch(
            &db,
            &[
                record("/m/a.flac", "fresh", "t-new"),
                record("/m/b.flac", "B", "t0"),
            ],
        )
        .unwrap();

        let all = rows(&db);
        assert_eq!(all.len(), 2);
        assert_eq!(all[0], (1, "/m/a.flac".into(), "fresh".into(), "t-new".into()));
    }

    /// The inverse shape: duplicates already present and no index to stop them, which is
    /// what the broken writer could theoretically have left behind. The upsert updates
    /// every match instead of failing.
    #[test]
    fn works_when_duplicate_paths_are_already_present() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        DATABASE_MIGRATIONS.to_latest(&mut conn).unwrap();
        conn.execute_batch(
            "INSERT INTO musics (path, title, modified_at) VALUES ('/m/a.flac', 'stale', 't-old');
             INSERT INTO musics (path, title, modified_at) VALUES ('/m/a.flac', 'dup', 't-older');",
        )
        .unwrap();
        let db = Database {
            conn: Mutex::new(conn),
        };

        upsert_music_batch(&db, &[record("/m/a.flac", "fresh", "t-new")]).unwrap();

        let all = rows(&db);
        assert_eq!(all.len(), 2, "no rows added");
        assert!(
            all.iter().all(|(_, _, title, _)| title == "fresh"),
            "every duplicate should have been updated, got {:?}",
            all
        );
    }

    /// Must survive a second launch, which re-runs the migrations against an
    /// already-current database.
    #[test]
    fn works_against_an_already_migrated_database() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        DATABASE_MIGRATIONS.to_latest(&mut conn).unwrap();
        DATABASE_MIGRATIONS.to_latest(&mut conn).unwrap();
        let db = Database {
            conn: Mutex::new(conn),
        };

        upsert_music_batch(&db, &[record("/m/a.flac", "A", "t0")]).unwrap();
        assert_eq!(rows(&db).len(), 1);
    }

    #[test]
    fn delete_music_paths_removes_only_listed() {
        let db = db();
        upsert_music_batch(
            &db,
            &[
                record("/m/a.flac", "A", "t0"),
                record("/m/b.flac", "B", "t0"),
            ],
        )
        .unwrap();

        let removed = delete_music_paths(&db, &["/m/a.flac".into(), "/m/zz.flac".into()]).unwrap();
        assert_eq!(removed, 1);
        assert_eq!(rows(&db), vec![(2, "/m/b.flac".into(), "B".into(), "t0".into())]);
    }
}
