//! UniFFI surface for macOS (Swift) and Android (Kotlin).
//!
//! One file per feature; every method is `<feature>_<verb>` and lives in the file
//! named after its feature (`player_seek` -> `player.rs`). Generated Swift/Kotlin
//! names are the camelCase form (`playerSeek`).

mod album;
mod artwork;
mod backdrop;
mod discord;
mod error;
mod events;
mod library;
mod lyrics;
mod player;
mod queue;

pub use error::FluyerError;
pub use events::{FluyerEvent, FluyerEventListener};

use crate::events::EventSink;
use crate::FluyerEngine;
use std::path::Path;
use std::sync::Arc;

#[derive(uniffi::Object)]
pub struct FluyerAppEngine {
    inner: Arc<FluyerEngine>,
}

#[uniffi::export]
impl FluyerAppEngine {
    #[uniffi::constructor]
    pub fn new(
        data_dir: String,
        cache_dir: String,
        listener: Option<Box<dyn FluyerEventListener>>,
    ) -> Result<Arc<Self>, FluyerError> {
        let sink = listener.map(|l| Arc::new(events::EventSinkBridge::new(l)) as Arc<dyn EventSink>);
        let engine = FluyerEngine::new(Path::new(&data_dir), Path::new(&cache_dir), sink)
            .map_err(|e| FluyerError::InitFailed { reason: e })?;
        Ok(Arc::new(Self { inner: Arc::new(engine) }))
    }
}
