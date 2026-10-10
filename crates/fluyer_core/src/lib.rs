#[macro_use]
pub mod logger;
pub mod audio;
pub mod db;
mod engine;
pub mod events;
pub mod c_api;
pub mod library;
pub mod metadata;
pub mod services;
pub mod uniffi_api;
pub mod view_models;

uniffi::setup_scaffolding!();

pub use engine::FluyerEngine;
