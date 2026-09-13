use rusqlite_migration::{Migrations, M};

pub const MIGRATIONS_SLICE: &[M<'_>] = &[
    M::up(
        "
    CREATE TABLE musics (
        id INTEGER PRIMARY KEY,
        path TEXT NOT NULL,
        duration INTEGER,

        title TEXT,
        artist TEXT,
        album TEXT,
        album_artist TEXT,
        track_number VARCHAR(10),
        genre TEXT,
        date TEXT,
        bits_per_sample INTEGER,
        sample_rate INTEGER,
        modified_at TEXT NOT NULL
    )",
    ),
    M::up(
        "
    CREATE TABLE playlists (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TRIGGER playlists_update_timestamp
    AFTER UPDATE ON playlists
    BEGIN
        UPDATE playlists SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

    CREATE TABLE playlist_musics (
        id INTEGER PRIMARY KEY,
        playlist_id INTEGER NOT NULL,
        music_id INTEGER NOT NULL,
        position INTEGER NOT NULL,
        FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
        FOREIGN KEY (music_id) REFERENCES musics(id) ON DELETE CASCADE
    );
    ",
    ),
    M::up(
        "
    DROP TABLE IF EXISTS playlist_musics;
    DROP TABLE IF EXISTS playlists;
    DROP TRIGGER IF EXISTS playlists_update_timestamp;

    CREATE TABLE playlists (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        image TEXT,
        title TEXT,
        artist TEXT,
        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TRIGGER playlists_update_timestamp
    AFTER UPDATE ON playlists
    BEGIN
        UPDATE playlists SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

    CREATE TABLE playlist_musics (
        id INTEGER PRIMARY KEY,
        playlist_id INTEGER NOT NULL,
        path TEXT NOT NULL,
        position INTEGER NOT NULL,
        FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE
    );
    ",
    ),
];

pub const DATABASE_MIGRATIONS: Migrations<'_> = Migrations::from_slice(MIGRATIONS_SLICE);
