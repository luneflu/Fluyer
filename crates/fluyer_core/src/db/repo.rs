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

    for record in records {
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
            ],
        );
    }

    tx.commit()
        .map_err(|e| format!("Transaction commit failed: {}", e))?;
    Ok(())
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
