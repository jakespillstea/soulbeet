use reqwest::Client;
use serde::Deserialize;
use shared::metadata::{Album, AlbumWithTracks, SearchResult, Track};
use tracing::{info, warn};

use crate::error::{Result, SoulseekError};

const LASTFM_API_BASE: &str = "https://ws.audioscrobbler.com/2.0/";

#[derive(Debug, Deserialize)]
struct LastFmImage {
    #[serde(rename = "#text")]
    url: String,
    size: String,
}

impl LastFmImage {
    fn get_largest(images: &[LastFmImage]) -> Option<String> {
        let sizes = ["extralarge", "large", "medium", "small"];
        for size in sizes {
            if let Some(img) = images.iter().find(|i| i.size == size) {
                if !img.url.is_empty() {
                    return Some(img.url.clone());
                }
            }
        }
        images.iter().find(|i| !i.url.is_empty()).map(|i| i.url.clone())
    }
}

#[derive(Debug, Deserialize)]
struct AlbumSearchResponse {
    results: AlbumSearchResults,
}

#[derive(Debug, Deserialize)]
struct AlbumSearchResults {
    #[serde(rename = "albummatches")]
    album_matches: AlbumMatches,
}

#[derive(Debug, Deserialize)]
struct AlbumMatches {
    album: Vec<LastFmAlbum>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LastFmAlbum {
    name: String,
    artist: String,
    url: String,
    #[serde(default)]
    mbid: Option<String>,
    #[serde(default)]
    image: Vec<LastFmImage>,
}

#[derive(Debug, Deserialize)]
struct TrackSearchResponse {
    results: TrackSearchResults,
}

#[derive(Debug, Deserialize)]
struct TrackSearchResults {
    #[serde(rename = "trackmatches")]
    track_matches: TrackMatches,
}

#[derive(Debug, Deserialize)]
struct TrackMatches {
    track: Vec<LastFmTrack>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LastFmTrack {
    name: String,
    artist: String,
    url: String,
    #[serde(default)]
    mbid: Option<String>,
    #[serde(default)]
    listeners: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AlbumInfoResponse {
    album: LastFmAlbumInfo,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LastFmAlbumInfo {
    name: String,
    artist: String,
    #[serde(default)]
    mbid: Option<String>,
    url: String,
    #[serde(default)]
    image: Vec<LastFmImage>,
    #[serde(default)]
    tracks: Option<LastFmTracks>,
    #[serde(default)]
    wiki: Option<LastFmWiki>,
}

#[derive(Debug, Deserialize)]
struct LastFmTracks {
    track: LastFmTrackList,
}

// Last.fm returns either a single track object or an array
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum LastFmTrackList {
    Single(Box<LastFmAlbumTrack>),
    Multiple(Vec<LastFmAlbumTrack>),
}

impl LastFmTrackList {
    fn into_vec(self) -> Vec<LastFmAlbumTrack> {
        match self {
            LastFmTrackList::Single(track) => vec![*track],
            LastFmTrackList::Multiple(tracks) => tracks,
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LastFmAlbumTrack {
    name: String,
    #[serde(default)]
    duration: Option<u32>,
    #[serde(default)]
    mbid: Option<String>,
    #[serde(default)]
    artist: Option<LastFmTrackArtist>,
    #[serde(rename = "@attr", default)]
    attr: Option<TrackAttr>,
}

#[derive(Debug, Deserialize)]
struct LastFmTrackArtist {
    name: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TrackAttr {
    rank: u32,
}

#[derive(Debug, Deserialize)]
struct LastFmWiki {
    #[serde(default)]
    published: Option<String>,
}

pub struct LastFmProvider {
    client: Client,
    api_key: String,
}

impl LastFmProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
        }
    }

    pub fn from_env() -> Option<Self> {
        std::env::var("LASTFM_API_KEY").ok().map(Self::new)
    }

    async fn search_albums_internal(
        &self,
        artist: Option<&str>,
        query: &str,
        limit: usize,
    ) -> Result<Vec<LastFmAlbum>> {
        let album_query = if let Some(artist) = artist {
            format!("{} {}", artist, query)
        } else {
            query.to_string()
        };

        let params = [
            ("method", "album.search"),
            ("album", &album_query),
            ("api_key", &self.api_key),
            ("format", "json"),
            ("limit", &limit.to_string()),
        ];

        let url = reqwest::Url::parse_with_params(LASTFM_API_BASE, &params).map_err(|e| {
            SoulseekError::Api {
                status: 500,
                message: format!("Failed to build URL: {}", e),
            }
        })?;

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| SoulseekError::Api {
                status: 500,
                message: format!("Last.fm request failed: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(SoulseekError::Api {
                status: response.status().as_u16(),
                message: format!("Last.fm API error: {}", response.status()),
            });
        }

        let data: AlbumSearchResponse =
            response.json().await.map_err(|e| SoulseekError::Api {
                status: 500,
                message: format!("Failed to parse Last.fm response: {}", e),
            })?;

        Ok(data.results.album_matches.album)
    }

    async fn search_tracks_internal(
        &self,
        artist: Option<&str>,
        query: &str,
        limit: usize,
    ) -> Result<Vec<LastFmTrack>> {
        let mut params = vec![
            ("method", "track.search".to_string()),
            ("track", query.to_string()),
            ("api_key", self.api_key.clone()),
            ("format", "json".to_string()),
            ("limit", limit.to_string()),
        ];

        if let Some(artist) = artist {
            params.push(("artist", artist.to_string()));
        }

        let url = reqwest::Url::parse_with_params(LASTFM_API_BASE, &params).map_err(|e| {
            SoulseekError::Api {
                status: 500,
                message: format!("Failed to build URL: {}", e),
            }
        })?;

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| SoulseekError::Api {
                status: 500,
                message: format!("Last.fm request failed: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(SoulseekError::Api {
                status: response.status().as_u16(),
                message: format!("Last.fm API error: {}", response.status()),
            });
        }

        let data: TrackSearchResponse =
            response.json().await.map_err(|e| SoulseekError::Api {
                status: 500,
                message: format!("Failed to parse Last.fm response: {}", e),
            })?;

        Ok(data.results.track_matches.track)
    }

    async fn get_album_info(&self, artist: &str, album: &str) -> Result<LastFmAlbumInfo> {
        let params = [
            ("method", "album.getInfo"),
            ("artist", artist),
            ("album", album),
            ("api_key", &self.api_key),
            ("format", "json"),
        ];

        let url = reqwest::Url::parse_with_params(LASTFM_API_BASE, &params).map_err(|e| {
            SoulseekError::Api {
                status: 500,
                message: format!("Failed to build URL: {}", e),
            }
        })?;

        info!("Fetching album info from Last.fm: {} - {}", artist, album);

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| SoulseekError::Api {
                status: 500,
                message: format!("Last.fm request failed: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(SoulseekError::Api {
                status: response.status().as_u16(),
                message: format!("Last.fm API error: {}", response.status()),
            });
        }

        let data: AlbumInfoResponse =
            response.json().await.map_err(|e| SoulseekError::Api {
                status: 500,
                message: format!("Failed to parse Last.fm response: {}", e),
            })?;

        Ok(data.album)
    }
}

fn format_duration(seconds: Option<u32>) -> Option<String> {
    seconds.map(|s| {
        let minutes = s / 60;
        let secs = s % 60;
        format!("{:02}:{:02}", minutes, secs)
    })
}

fn generate_lastfm_id(artist: &str, name: &str) -> String {
    format!("lastfm:{}:{}", artist.to_lowercase(), name.to_lowercase())
}

#[async_trait::async_trait]
impl crate::MetadataProvider for LastFmProvider {
    fn id(&self) -> &'static str {
        "lastfm"
    }

    fn name(&self) -> &'static str {
        "Last.fm"
    }

    async fn search_albums(
        &self,
        artist: Option<&str>,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let albums = self.search_albums_internal(artist, query, limit).await?;

        Ok(albums
            .into_iter()
            .map(|a| {
                let mbid = a.mbid.filter(|s| !s.is_empty());
                let id = mbid
                    .clone()
                    .unwrap_or_else(|| generate_lastfm_id(&a.artist, &a.name));
                let cover_url = LastFmImage::get_largest(&a.image);
                SearchResult::Album(Album {
                    id,
                    title: a.name,
                    artist: a.artist,
                    release_date: None,
                    mbid,
                    cover_url,
                })
            })
            .collect())
    }

    async fn search_tracks(
        &self,
        artist: Option<&str>,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let tracks = self.search_tracks_internal(artist, query, limit).await?;

        Ok(tracks
            .into_iter()
            .map(|t| {
                let mbid = t.mbid.filter(|s| !s.is_empty());
                let id = mbid
                    .clone()
                    .unwrap_or_else(|| generate_lastfm_id(&t.artist, &t.name));
                SearchResult::Track(Track {
                    id,
                    title: t.name,
                    artist: t.artist,
                    album_id: None,
                    album_title: None,
                    release_date: None,
                    duration: None,
                    mbid,
                    release_mbid: None,
                })
            })
            .collect())
    }

    async fn get_album(&self, id: &str) -> Result<AlbumWithTracks> {
        if let Some(rest) = id.strip_prefix("lastfm:") {
            if let Some((artist, album)) = rest.split_once(':') {
                let info = self.get_album_info(artist, album).await?;

                let album_mbid = info.mbid.as_ref().filter(|s| !s.is_empty()).cloned();
                let cover_url = LastFmImage::get_largest(&info.image);

                let tracks = info
                    .tracks
                    .map(|t| {
                        t.track
                            .into_vec()
                            .into_iter()
                            .map(|track| {
                                let track_mbid = track.mbid.filter(|s| !s.is_empty());
                                Track {
                                    id: generate_lastfm_id(&info.artist, &track.name),
                                    title: track.name,
                                    artist: track
                                        .artist
                                        .map(|a| a.name)
                                        .unwrap_or_else(|| info.artist.clone()),
                                    album_id: Some(id.to_string()),
                                    album_title: Some(info.name.clone()),
                                    release_date: info.wiki.as_ref().and_then(|w| w.published.clone()),
                                    duration: format_duration(track.duration),
                                    mbid: track_mbid,
                                    release_mbid: album_mbid.clone(),
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                return Ok(AlbumWithTracks {
                    album: Album {
                        id: id.to_string(),
                        title: info.name,
                        artist: info.artist,
                        release_date: info.wiki.and_then(|w| w.published),
                        mbid: album_mbid,
                        cover_url,
                    },
                    tracks,
                });
            }
        }

        warn!("Cannot fetch album by MBID from Last.fm: {}", id);
        Err(SoulseekError::Api {
            status: 400,
            message: "Last.fm requires artist:album format, not MBID".to_string(),
        })
    }
}
