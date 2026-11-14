//! Values the provider hands to the UI. Deliberately provider-shaped, not the local
//! library's `TrackItemViewModel`: remote items have no file path and no library index.

use serde::{Deserialize, Serialize};

#[derive(uniffi::Enum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchItemKind {
    Track,
    Album,
    Artist,
    Playlist,
}

#[derive(uniffi::Record, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchItem {
    pub title: String,
    pub subtitle: Option<String>,
    pub kind: SearchItemKind,
    pub browse_id: Option<String>,
    pub video_id: Option<String>,
    pub thumbnail: Option<String>,
}

/// One page of the home feed: shelves plus the token for the next page.
#[derive(uniffi::Record, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HomeFeed {
    pub shelves: Vec<FeedShelf>,
    pub continuation: Option<String>,
}

/// One horizontally scrolling row of the home feed.
#[derive(uniffi::Record, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeedShelf {
    pub title: String,
    pub items: Vec<SearchItem>,
}

#[derive(uniffi::Record, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlbumRef {
    pub browse_id: String,
    pub name: String,
}

#[derive(uniffi::Record, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackRef {
    pub title: String,
    pub video_id: Option<String>,
    pub duration: Option<String>,
}

#[derive(uniffi::Record, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlbumDetail {
    pub browse_id: String,
    pub name: String,
    pub artist: Option<String>,
    pub year: Option<String>,
    pub thumbnail: Option<String>,
    pub tracks: Vec<TrackRef>,
}

#[derive(uniffi::Record, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamFormat {
    pub mime_type: Option<String>,
    pub bitrate: Option<u64>,
    /// Direct URL when YouTube returned one (no cipher involved).
    pub url: Option<String>,
    /// `s=…&url=…` when the URL still needs the player's cipher; handed onward, not solved here.
    pub signature_cipher: Option<String>,
    pub expires_in_seconds: Option<u64>,
}

#[derive(uniffi::Record, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamInfo {
    pub formats: Vec<StreamFormat>,
    pub playability_status: Option<String>,
    pub reason: Option<String>,
}
