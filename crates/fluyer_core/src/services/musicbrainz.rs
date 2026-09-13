use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://musicbrainz.org/ws/2";
const BASE_COVER_ART_URL: &str = "https://coverartarchive.org";

#[derive(Serialize, Deserialize)]
struct ReleaseGroupResponse {
    #[serde(alias = "release-groups")]
    release_groups: Vec<ReleaseGroup>,
}

#[derive(Serialize, Deserialize)]
struct ReleaseResponse {
    releases: Vec<Release>,
}

#[derive(Serialize, Deserialize)]
struct ReleaseGroup {
    id: String,
}

#[derive(Serialize, Deserialize)]
struct Release {
    id: String,
}

#[derive(Serialize, Deserialize)]
struct CoverArtResponse {
    images: Vec<CoverArtImage>,
}

#[derive(Serialize, Deserialize)]
struct CoverArtImage {
    thumbnails: CoverArtThumbnails,
}

#[derive(Serialize, Deserialize)]
struct CoverArtThumbnails {
    #[serde(alias = "500")]
    i500: String,
}

pub struct MusicBrainz;

impl MusicBrainz {
    pub async fn get_cover_art_url(
        artist: &str,
        album: Option<&str>,
        title: Option<&str>,
    ) -> Result<Option<String>, String> {
        let (btype, bquery) = if let Some(alb) = album {
            ("release-group", format!("{} {}", artist, alb))
        } else if let Some(tit) = title {
            ("release", format!("{} {}", artist, tit))
        } else {
            return Ok(None);
        };

        crate::flog!("MusicBrainz", "{} request: '{}'", btype, bquery);

        let client = reqwest::Client::builder()
            .user_agent("Fluyer/1.0 ( https://github.com/alvindimas05/fluyer )")
            .build()
            .map_err(|e| e.to_string())?;

        let url = format!("{}/{}?query={}&fmt=json&limit=1", BASE_URL, btype, urlencoding::encode(&bquery));
        let response = client.get(&url).send().await.map_err(|e| e.to_string())?;

        let id = if album.is_some() {
            let parsed = response.json::<ReleaseGroupResponse>().await.map_err(|e| e.to_string())?;
            parsed.release_groups.into_iter().next().map(|rg| rg.id)
        } else {
            let parsed = response.json::<ReleaseResponse>().await.map_err(|e| e.to_string())?;
            parsed.releases.into_iter().next().map(|r| r.id)
        };

        let Some(target_id) = id else {
            return Ok(None);
        };

        let ca_url = format!("{}/{}/{}", BASE_COVER_ART_URL, btype, target_id);
        let ca_res = client.get(&ca_url).send().await.map_err(|e| e.to_string())?;

        if ca_res.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let ca_json = ca_res.json::<CoverArtResponse>().await.map_err(|e| e.to_string())?;
        Ok(ca_json.images.into_iter().next().map(|img| img.thumbnails.i500))
    }
}
