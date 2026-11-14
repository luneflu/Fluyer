//! Tolerant reads of InnerTube JSON.
//!
//! Every lookup is optional and key-driven: an unexpected shape drops that field
//! instead of failing the response. That tolerance, not a scripting layer, is what
//! makes YouTube Music schema changes survivable — see `services/profile.rs` for the
//! part that *is* data (client version, keys, endpoint paths).

use super::models::{AlbumDetail, AlbumRef, FeedShelf, SearchItem, SearchItemKind, StreamFormat, StreamInfo, TrackRef};
use serde_json::Value;

/// Walks `path` through objects, then array indices given as digits.
pub fn nav<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for step in path {
        current = match current {
            Value::Object(map) => map.get(*step)?,
            Value::Array(items) => items.get(step.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(current)
}

/// Text of a `{simpleText}` / `{runs:[{text}]}` / plain string node.
pub fn text(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        _ => nav(value, &["simpleText"])
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                let runs = nav(value, &["runs"])?.as_array()?;
                let joined: String = runs.iter().filter_map(|r| nav(r, &["text"])?.as_str()).collect();
                (!joined.is_empty()).then_some(joined)
            }),
    }
}

/// Concatenated run text at a path, e.g. the "Album • Artist • 2024" subtitle.
pub fn runs_text(value: &Value, path: &[&str]) -> Option<String> {
    text(nav(value, path)?)
}

fn best_thumbnail(value: &Value) -> Option<String> {
    // Feed cards (`musicTwoRowItemRenderer`) carry the image under `thumbnailRenderer`
    // instead of `thumbnail`; search rows use the latter.
    let thumbs = nav(value, &["thumbnail", "musicThumbnailRenderer", "thumbnail", "thumbnails"])
        .or_else(|| nav(value, &["thumbnailRenderer", "musicThumbnailRenderer", "thumbnail", "thumbnails"]))
        .or_else(|| nav(value, &["thumbnail", "thumbnails"]))?
        .as_array()?;
    thumbs
        .iter()
        .max_by_key(|t| nav(t, &["width"]).and_then(Value::as_u64).unwrap_or(0))
        .and_then(|t| nav(t, &["url"]).and_then(Value::as_str))
        .map(str::to_string)
}

fn video_id(renderer: &Value) -> Option<String> {
    nav(renderer, &["playlistItemData", "videoId"])
        .and_then(Value::as_str)
        .or_else(|| nav(renderer, &["overlay", "musicItemThumbnailOverlayRenderer", "content", "musicPlayButtonRenderer", "playNavigationEndpoint", "watchEndpoint", "videoId"]).and_then(Value::as_str))
        .or_else(|| nav(renderer, &["navigationEndpoint", "watchEndpoint", "videoId"]).and_then(Value::as_str))
        .map(str::to_string)
}

fn browse_id(renderer: &Value) -> Option<String> {
    nav(renderer, &["navigationEndpoint", "browseEndpoint", "browseId"])
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn subtitle(renderer: &Value) -> Option<String> {
    runs_text(renderer, &["subtitle"]) .or_else(|| runs_text(renderer, &["flexColumns", "1", "musicResponsiveListItemFlexColumnRenderer", "text"]))
        .or_else(|| runs_text(renderer, &["flexColumns", "2", "musicResponsiveListItemFlexColumnRenderer", "text"]))
}

fn title(renderer: &Value) -> Option<String> {
    runs_text(renderer, &["title"]).or_else(|| {
        runs_text(
            renderer,
            &["flexColumns", "0", "musicResponsiveListItemFlexColumnRenderer", "text"],
        )
    })
}

/// `musicResponsiveListItemRenderer` nodes anywhere under the results shelf.
fn shelf_items(root: &Value) -> Vec<&Value> {
    let mut out = Vec::new();
    collect_named(root, "musicResponsiveListItemRenderer", &mut out);
    out
}

/// Either list renderer, in render order: `musicTwoRowItemRenderer` for album/playlist
/// cards, `musicResponsiveListItemRenderer` for song rows (the "Quick picks" shelf).
fn collect_items<'a>(value: &'a Value, out: &mut Vec<&'a Value>) {
    match value {
        Value::Object(map) => {
            for (name, child) in map {
                if name == "musicTwoRowItemRenderer" || name == "musicResponsiveListItemRenderer" {
                    out.push(child);
                }
                collect_items(child, out);
            }
        }
        Value::Array(items) => items.iter().for_each(|item| collect_items(item, out)),
        _ => {}
    }
}

fn collect_named<'a>(value: &'a Value, key: &str, out: &mut Vec<&'a Value>) {
    match value {
        Value::Object(map) => {
            for (name, child) in map {
                if name == key {
                    out.push(child);
                }
                collect_named(child, key, out);
            }
        }
        Value::Array(items) => items.iter().for_each(|item| collect_named(item, key, out)),
        _ => {}
    }
}

fn classify(page_type: Option<&str>, subtitle: Option<&str>) -> SearchItemKind {
    match page_type {
        Some("MUSIC_PAGE_TYPE_ALBUM") => SearchItemKind::Album,
        Some("MUSIC_PAGE_TYPE_ARTIST") => SearchItemKind::Artist,
        Some("MUSIC_PAGE_TYPE_PLAYLIST") => SearchItemKind::Playlist,
        Some(_) => SearchItemKind::Track,
        // Subtitle looks like "Artist • Album • 2021": classify on whole segments, so a
        // song of an album called "Album" is still a song.
        None => subtitle
            .unwrap_or_default()
            .split('•')
            .rev()
            .map(|segment| segment.trim().to_lowercase())
            .find_map(|segment| match segment.as_str() {
                "album" | "single" | "ep" => Some(SearchItemKind::Album),
                "artist" => Some(SearchItemKind::Artist),
                "playlist" => Some(SearchItemKind::Playlist),
                "song" | "video" => Some(SearchItemKind::Track),
                _ => None,
            })
            .unwrap_or(SearchItemKind::Track),
    }
}

/// One list renderer turned into the UI's item shape.
fn item_of(renderer: &Value) -> Option<SearchItem> {
    let title = title(renderer)?;
    let page_type = nav(renderer, &["navigationEndpoint", "browseEndpoint", "browseEndpointContextSupportedConfigs", "browseEndpointContextMusicConfig", "pageType"])
        .and_then(Value::as_str);
    let sub = subtitle(renderer);
    let kind = classify(page_type, sub.as_deref());
    Some(SearchItem {
        title,
        subtitle: sub,
        kind,
        browse_id: browse_id(renderer),
        video_id: video_id(renderer),
        thumbnail: best_thumbnail(renderer),
    })
}

/// Search results, in the order YouTube Music rendered them.
pub fn parse_search(body: &Value) -> Vec<SearchItem> {
    shelf_items(body).into_iter().filter_map(item_of).collect()
}

/// Home feed: every carousel shelf in render order. Shelves with no usable item are
/// dropped, since the UI would render an empty row.
pub fn parse_home(body: &Value) -> Vec<FeedShelf> {
    let mut carousels = Vec::new();
    collect_named(body, "musicCarouselShelfRenderer", &mut carousels);
    carousels
        .into_iter()
        .filter_map(|carousel| {
            let title = nav(carousel, &["header", "musicCarouselShelfBasicHeaderRenderer", "title"])
                .and_then(text)
                .or_else(|| find_first(carousel, "musicCarouselShelfBasicHeaderRenderer").and_then(|h| runs_text(h, &["title"])))?;
            let mut renderers = Vec::new();
            collect_items(carousel, &mut renderers);
            let items: Vec<SearchItem> = renderers.into_iter().filter_map(item_of).collect();
            if items.is_empty() {
                return None;
            }
            Some(FeedShelf { title, items })
        })
        .collect()
}

/// Album page: header (name/artist/year/cover) plus its tracks.
pub fn parse_album(body: &Value, browse_id: &str) -> Option<AlbumDetail> {
    let header = find_first(body, "musicResponsiveHeaderRenderer")
        .or_else(|| find_first(body, "musicDetailHeaderRenderer"))
        .or_else(|| find_first(body, "musicImmersiveHeaderRenderer"))?;
    let name = runs_text(header, &["title"])?;
    let artist = nav(header, &["straplineTextOne"])
        .and_then(|v| text(v))
        .or_else(|| runs_text(header, &["subtitle"]));
    let year = runs_text(header, &["subtitle"]).as_deref().and_then(extract_year);
    let tracks: Vec<TrackRef> = shelf_items(body)
        .into_iter()
        .filter_map(|renderer| {
            Some(TrackRef {
                title: title(renderer)?,
                video_id: video_id(renderer),
                duration: runs_text(renderer, &["fixedColumns", "0", "musicResponsiveListItemFixedColumnRenderer", "text"]),
            })
        })
        .collect();
    Some(AlbumDetail {
        browse_id: browse_id.to_string(),
        name,
        artist,
        year,
        thumbnail: best_thumbnail(header),
        tracks,
    })
}

/// Album/artist search hit → the album page fetch key.
pub fn album_ref(item: &SearchItem) -> Option<AlbumRef> {
    let browse_id = item.browse_id.clone()?;
    Some(AlbumRef { browse_id, name: item.title.clone() })
}

fn find_first<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    match value {
        Value::Object(map) => {
            if let Some(found) = map.get(key) {
                return Some(found);
            }
            map.values().find_map(|child| find_first(child, key))
        }
        Value::Array(items) => items.iter().find_map(|child| find_first(child, key)),
        _ => None,
    }
}

fn extract_year(text: &str) -> Option<String> {
    text.split(|c: char| !c.is_ascii_digit())
        .find(|part| part.len() == 4)
        .map(str::to_string)
}

/// Plain lyric text from a `browse` response: description runs, or the timed model.
pub fn parse_lyrics(body: &Value) -> Option<String> {
    if let Some(model) = find_first(body, "timedLyricsModel") {
        if let Some(lines) = nav(model, &["lyricsData", "timedLyricsData"]).and_then(Value::as_array) {
            let joined: Vec<&str> = lines
                .iter()
                .filter_map(|line| nav(line, &["lyricLine"]).and_then(Value::as_str))
                .collect();
            if !joined.is_empty() {
                return Some(joined.join("\n"));
            }
        }
    }
    if let Some(description) = find_first(body, "description") {
        if let Some(text) = text(description) {
            return Some(text);
        }
    }
    find_first(body, "lyricsText").and_then(text)
}

/// Streaming formats for a `player` response. Cipher-solved URLs are not produced
/// here: formats expose either `url` or the cipher fields the resolver needs.
pub fn parse_streaming(body: &Value) -> Option<StreamInfo> {
    let data = nav(body, &["streamingData"])?;
    let mut formats: Vec<StreamFormat> = Vec::new();
    for section in ["adaptiveFormats", "formats"] {
        let Some(items) = nav(data, &[section]).and_then(Value::as_array) else { continue };
        for item in items {
            formats.push(StreamFormat {
                mime_type: nav(item, &["mimeType"]).and_then(Value::as_str).map(str::to_string),
                bitrate: nav(item, &["bitrate"]).and_then(Value::as_u64),
                url: nav(item, &["url"]).and_then(Value::as_str).map(str::to_string),
                signature_cipher: nav(item, &["signatureCipher"]).and_then(Value::as_str).map(str::to_string),
                expires_in_seconds: nav(item, &["expiresInSeconds"])
                    .and_then(Value::as_str)
                    .and_then(|s| s.parse().ok()),
            });
        }
    }
    (!formats.is_empty()).then_some(StreamInfo {
        formats,
        playability_status: nav(body, &["playabilityStatus", "status"]).and_then(Value::as_str).map(str::to_string),
        reason: nav(body, &["playabilityStatus", "reason"]).and_then(Value::as_str).map(str::to_string),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn nav_and_text_tolerate_both_shapes() {
        let value = json!({"a": {"runs": [{"text": "Hi "}, {"text": "there"}]}, "b": {"simpleText": "x"}});
        assert_eq!(text(nav(&value, &["a"]).unwrap()).unwrap(), "Hi there");
        assert_eq!(text(nav(&value, &["b"]).unwrap()).unwrap(), "x");
        assert!(nav(&value, &["missing"]).is_none());
        assert!(nav(&value, &["a", "9"]).is_none());
    }

    #[test]
    fn search_finds_song_album_and_artist_rows() {
        let body = json!({
            "contents": {"x": {"musicShelfRenderer": {"contents": [
                {"musicResponsiveListItemRenderer": {
                    "playlistItemData": {"videoId": "v1"},
                    "flexColumns": [
                        {"musicResponsiveListItemFlexColumnRenderer": {"text": {"runs": [{"text": "Song A"}]}}},
                        {"musicResponsiveListItemFlexColumnRenderer": {"text": {"runs": [{"text": "Artist • Song"}]}}}
                    ],
                    "thumbnail": {"thumbnails": [{"width": 60, "url": "small"}, {"width": 544, "url": "big"}]}
                }},
                {"musicResponsiveListItemRenderer": {
                    "navigationEndpoint": {"browseEndpoint": {"browseId": "MPREb_album",
                        "browseEndpointContextSupportedConfigs": {"browseEndpointContextMusicConfig": {"pageType": "MUSIC_PAGE_TYPE_ALBUM"}}}},
                    "flexColumns": [
                        {"musicResponsiveListItemFlexColumnRenderer": {"text": {"runs": [{"text": "Album B"}]}}},
                        {"musicResponsiveListItemFlexColumnRenderer": {"text": {"runs": [{"text": "2024"}]}}}
                    ]
                }}
            ]}}}
        });
        let items = parse_search(&body);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].kind, SearchItemKind::Track);
        assert_eq!(items[0].video_id.as_deref(), Some("v1"));
        assert_eq!(items[0].thumbnail.as_deref(), Some("big"));
        assert_eq!(items[1].kind, SearchItemKind::Album);
        assert_eq!(album_ref(&items[1]).unwrap().browse_id, "MPREb_album");
    }

    #[test]
    fn album_header_and_tracks_parse_without_unknown_keys_failing() {
        let body = json!({
            "header": {"musicResponsiveHeaderRenderer": {
                "title": {"runs": [{"text": "Between Broken"}]},
                "subtitle": {"runs": [{"text": "Dabin • Album • 2021"}]},
                "thumbnail": {"musicThumbnailRenderer": {"thumbnail": {"thumbnails": [
                    {"width": 120, "url": "a"}, {"width": 1200, "url": "b"}]}}}
            }},
            "contents": {"musicShelfRenderer": {"contents": [
                {"musicResponsiveListItemRenderer": {
                    "playlistItemData": {"videoId": "v1"},
                    "flexColumns": [{"musicResponsiveListItemFlexColumnRenderer": {"text": {"runs": [{"text": "In Flames"}]}}}],
                    "fixedColumns": [{"musicResponsiveListItemFixedColumnRenderer": {"text": {"runs": [{"text": "4:12"}]}}}]
                }}
            ]}}
        });
        let album = parse_album(&body, "MPREb_1").unwrap();
        assert_eq!(album.name, "Between Broken");
        assert_eq!(album.year.as_deref(), Some("2021"));
        assert_eq!(album.thumbnail.as_deref(), Some("b"));
        assert_eq!(album.tracks.len(), 1);
        assert_eq!(album.tracks[0].duration.as_deref(), Some("4:12"));
    }

    #[test]
    fn home_collects_carousel_shelves_and_drops_empty_ones() {
        let body = json!({
            "contents": {"singleColumnBrowseResultsRenderer": {"tabs": [{"tabRenderer": {"content": {
                "sectionListRenderer": {"contents": [
                    {"musicCarouselShelfRenderer": {
                        "header": {"musicCarouselShelfBasicHeaderRenderer": {"title": {"runs": [{"text": "Quick picks"}]}}},
                        "contents": [
                            {"musicTwoRowItemRenderer": {
                                "title": {"runs": [{"text": "Song A"}]},
                                "subtitle": {"runs": [{"text": "Artist • Song"}]},
                                "navigationEndpoint": {"watchEndpoint": {"videoId": "v1"}},
                                "thumbnail": {"thumbnails": [{"width": 120, "url": "small"}, {"width": 544, "url": "big"}]}
                            }},
                            {"musicTwoRowItemRenderer": {
                                "title": {"runs": [{"text": "Album B"}]},
                                "subtitle": {"runs": [{"text": "Artist • Album"}]},
                                "navigationEndpoint": {"browseEndpoint": {"browseId": "MPREb_2",
                                    "browseEndpointContextSupportedConfigs": {"browseEndpointContextMusicConfig": {"pageType": "MUSIC_PAGE_TYPE_ALBUM"}}}},
                                "thumbnailRenderer": {"musicThumbnailRenderer": {"thumbnail": {"thumbnails": [
                                    {"width": 192, "url": "row-small"}, {"width": 576, "url": "row-big"}]}}}
                            }},
                            {"musicResponsiveListItemRenderer": {
                                "playlistItemData": {"videoId": "v3"},
                                "flexColumns": [
                                    {"musicResponsiveListItemFlexColumnRenderer": {"text": {"runs": [{"text": "Song C"}]}}},
                                    {"musicResponsiveListItemFlexColumnRenderer": {"text": {"runs": [{"text": "Artist"}]}}}
                                ]
                            }}
                        ]
                    }},
                    {"musicCarouselShelfRenderer": {
                        "header": {"musicCarouselShelfBasicHeaderRenderer": {"title": {"runs": [{"text": "Empty"}]}}},
                        "contents": []
                    }}
                ]}
            }}}]}}
        });
        let shelves = parse_home(&body);
        assert_eq!(shelves.len(), 1);
        assert_eq!(shelves[0].title, "Quick picks");
        assert_eq!(shelves[0].items.len(), 3);
        assert_eq!(shelves[0].items[0].video_id.as_deref(), Some("v1"));
        assert_eq!(shelves[0].items[0].thumbnail.as_deref(), Some("big"));
        assert_eq!(shelves[0].items[1].kind, SearchItemKind::Album);
        assert_eq!(shelves[0].items[1].browse_id.as_deref(), Some("MPREb_2"));
        assert_eq!(shelves[0].items[1].thumbnail.as_deref(), Some("row-big"));
        assert_eq!(shelves[0].items[2].title, "Song C");
        assert_eq!(shelves[0].items[2].video_id.as_deref(), Some("v3"));
    }

    #[test]
    fn lyrics_prefer_timed_lines_then_description() {
        let timed = json!({"a": {"timedLyricsModel": {"lyricsData": {"timedLyricsData": [
            {"lyricLine": "line one"}, {"lyricLine": "line two"}]}}}});
        assert_eq!(parse_lyrics(&timed).unwrap(), "line one\nline two");
        let description = json!({"b": {"description": {"runs": [{"text": "plain lyrics"}]}}});
        assert_eq!(parse_lyrics(&description).unwrap(), "plain lyrics");
        assert!(parse_lyrics(&json!({})).is_none());
    }

    #[test]
    fn streaming_formats_expose_urls_or_cipher_fields() {
        let body = json!({"streamingData": {
            "adaptiveFormats": [
                {"mimeType": "audio/mp4; codecs=\"mp4a.40.2\"", "bitrate": 129000, "url": "https://x", "expiresInSeconds": "21540"},
                {"mimeType": "audio/webm; codecs=\"opus\"", "bitrate": 160000, "signatureCipher": "s=abc&url=https%3A%2F%2Fy"}
            ],
            "playabilityStatus": "OK"
        }, "playabilityStatus": {"status": "OK"}});
        let info = parse_streaming(&body).unwrap();
        assert_eq!(info.formats.len(), 2);
        assert_eq!(info.formats[0].expires_in_seconds, Some(21540));
        assert_eq!(info.formats[0].url.as_deref(), Some("https://x"));
        assert!(info.formats[1].url.is_none());
        assert!(info.formats[1].signature_cipher.is_some());
        assert_eq!(info.playability_status.as_deref(), Some("OK"));
    }
}
