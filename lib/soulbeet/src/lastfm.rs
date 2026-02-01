use reqwest::Client;
use serde::Deserialize;
use shared::musicbrainz::{Album, AlbumWithTracks, SearchResult, Track};
use tracing::{info, warn};

use crate::error::{Result, SoulseekError};

const LASTFM_API_BASE: &str = "https://ws.audioscrobbler.com/2.0/";

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

/// Generate a unique ID for Last.fm items (they don't always have MBIDs)
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
                SearchResult::Album(Album {
                    id: a.mbid.unwrap_or_else(|| generate_lastfm_id(&a.artist, &a.name)),
                    title: a.name,
                    artist: a.artist,
                    release_date: None, // Last.fm search doesn't return release dates
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
                SearchResult::Track(Track {
                    id: t.mbid.unwrap_or_else(|| generate_lastfm_id(&t.artist, &t.name)),
                    title: t.name,
                    artist: t.artist,
                    album_id: None,
                    album_title: None,
                    release_date: None,
                    duration: None,
                })
            })
            .collect())
    }

    async fn get_album(&self, id: &str) -> Result<AlbumWithTracks> {
        // Last.fm IDs are in format "lastfm:artist:album" or MBIDs
        // For MBID lookups, we'd need to use MusicBrainz - Last.fm needs artist+album
        if let Some(rest) = id.strip_prefix("lastfm:") {
            if let Some((artist, album)) = rest.split_once(':') {
                let info = self.get_album_info(artist, album).await?;

                let tracks = info
                    .tracks
                    .map(|t| {
                        t.track
                            .into_vec()
                            .into_iter()
                            .map(|track| Track {
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
                    },
                    tracks,
                });
            }
        }

        // If it's an MBID, we can't fetch it from Last.fm directly
        // The caller should use MusicBrainz for MBID lookups
        warn!("Cannot fetch album by MBID from Last.fm: {}", id);
        Err(SoulseekError::Api {
            status: 400,
            message: "Last.fm requires artist:album format, not MBID".to_string(),
        })
    }
}
