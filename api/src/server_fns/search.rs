use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use shared::{
    download::{DownloadQuery, SearchResult as DownloadSearchResult},
    musicbrainz::{AlbumWithTracks, SearchResult},
};

#[cfg(feature = "server")]
use crate::{server_fns::server_error, AuthSession};
#[cfg(feature = "server")]
use crate::services::{download_backend, metadata_provider};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchQuery {
    pub artist: Option<String>,
    pub query: String,
    #[serde(default)]
    pub provider: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlbumQuery {
    pub id: String,
    #[serde(default)]
    pub provider: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PollQuery {
    pub search_id: String,
    #[serde(default)]
    pub backend: Option<String>,
}

#[post("/api/metadata/search/album", _: AuthSession)]
pub async fn search_album(input: SearchQuery) -> Result<Vec<SearchResult>, ServerFnError> {
    let provider = metadata_provider(input.provider.as_deref())
        .await
        .map_err(server_error)?;

    provider
        .search_albums(input.artist.as_deref(), &input.query, 25)
        .await
        .map_err(server_error)
}

#[post("/api/metadata/search/track", _: AuthSession)]
pub async fn search_track(input: SearchQuery) -> Result<Vec<SearchResult>, ServerFnError> {
    let provider = metadata_provider(input.provider.as_deref())
        .await
        .map_err(server_error)?;

    provider
        .search_tracks(input.artist.as_deref(), &input.query, 25)
        .await
        .map_err(server_error)
}

#[post("/api/metadata/album", _: AuthSession)]
pub async fn find_album(input: AlbumQuery) -> Result<AlbumWithTracks, ServerFnError> {
    let provider = metadata_provider(input.provider.as_deref())
        .await
        .map_err(server_error)?;

    provider.get_album(&input.id).await.map_err(server_error)
}

#[post("/api/download/search/start", _: AuthSession)]
pub async fn start_download_search(data: DownloadQuery) -> Result<String, ServerFnError> {
    let backend = download_backend(data.backend.as_deref())
        .await
        .map_err(|e| server_error(format!("download backend not available: {}", e)))?;

    backend
        .start_search(data.album.as_ref(), &data.tracks)
        .await
        .map_err(server_error)
}

#[post("/api/download/search/poll", _: AuthSession)]
pub async fn poll_download_search(input: PollQuery) -> Result<DownloadSearchResult, ServerFnError> {
    let backend = download_backend(input.backend.as_deref())
        .await
        .map_err(|e| server_error(format!("download backend not available: {}", e)))?;

    backend
        .poll_search(&input.search_id)
        .await
        .map_err(server_error)
}
