pub mod migrations;
pub mod repo;

use migrations::DATABASE_MIGRATIONS;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub const DATABASE_NAME: &str = "fluyer.db";

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn open(db_path: &Path) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let mut conn = Connection::open(db_path)
            .map_err(|e| format!("Failed to open database at {:?}: {}", db_path, e))?;

        conn.pragma_update_and_check(None, "journal_mode", "WAL", |_| Ok(()))
            .map_err(|e| format!("Failed to set WAL journal mode: {}", e))?;

        DATABASE_MIGRATIONS
            .to_latest(&mut conn)
            .map_err(|e| format!("Failed to run database migrations: {}", e))?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}
