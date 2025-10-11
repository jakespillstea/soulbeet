//! This crate contains all shared fullstack server functions.

use chrono::Duration;
use dioxus::logger::tracing::info;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use shared::musicbrainz::{AlbumWithTracks, SearchResult};
#[cfg(feature = "server")]
use soulful::musicbrainz;
#[cfg(feature = "server")]
use soulful::slskd::SoulseekClientBuilder;

#[cfg(feature = "server")]
async fn slskd_search(input: &str) -> String {
    let client = SoulseekClientBuilder::new()
        .api_key("BOVeIS961OlDWlUeEjF6DsIZKzf857ijKBGFWWw4N9Scj1xwoq2C3VbjMBU=")
        .base_url("http://192.168.1.105:5030/")
        .download_path("/tmp/downloads")
        .build()
        .unwrap();

    let health = client.check_connection().await;
    let search = client.search(&input, Duration::seconds(30)).await;

    let mut tracks = vec![];

    for result in search.iter() {
        for track in &result.0 {
            tracks.push(track.base.filename.clone());
        }
    }

    info!("{search:?}");
    format!("Connection: {health}\nSearch: {}", tracks.join(" | "))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchQuery {
    pub artist: Option<String>,
    pub query: String,
}

#[server]
pub async fn search_album(input: SearchQuery) -> Result<Vec<SearchResult>, ServerFnError> {
    let results = musicbrainz::search(
        &input.artist,
        &input.query,
        musicbrainz::SearchType::Album,
        10,
    )
    .await?;

    Ok(results)
}

#[server]
pub async fn search_track(input: SearchQuery) -> Result<Vec<SearchResult>, ServerFnError> {
    let results = musicbrainz::search(
        &input.artist,
        &input.query,
        musicbrainz::SearchType::Track,
        10,
    )
    .await?;

    Ok(results)
}

#[server]
pub async fn find_album(id: String) -> Result<AlbumWithTracks, ServerFnError> {
    let results = musicbrainz::find_album(&id).await?;

    Ok(results)
}
