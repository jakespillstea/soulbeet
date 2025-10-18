//! This crate contains all shared fullstack server functions.

use std::sync::LazyLock;

use chrono::Duration;
use dioxus::{
    logger::tracing::info,
    prelude::{server_fn::error::NoCustomError, *},
};
use serde::{Deserialize, Serialize};
use shared::{
    download::DownloadQuery,
    musicbrainz::{AlbumWithTracks, SearchResult},
    slskd::AlbumResult,
};

#[cfg(feature = "server")]
use shared::musicbrainz::Track;
#[cfg(feature = "server")]
use soulful::musicbrainz;
#[cfg(feature = "server")]
use soulful::slskd::{SoulseekClient, SoulseekClientBuilder};

#[cfg(feature = "server")]
static SLSKD_CLIENT: LazyLock<SoulseekClient> = LazyLock::new(|| {
    SoulseekClientBuilder::new()
        .api_key("BOVeIS961OlDWlUeEjF6DsIZKzf857ijKBGFWWw4N9Scj1xwoq2C3VbjMBU=")
        .base_url("http://192.168.1.105:5030/")
        .download_path("/tmp/downloads")
        .build()
        .unwrap()
});

#[cfg(feature = "server")]
async fn slskd_search(
    artist: String,
    album: String,
    tracks: Vec<Track>,
) -> Result<Vec<AlbumResult>, ServerFnError> {
    let health = SLSKD_CLIENT.check_connection().await;
    let mut search = SLSKD_CLIENT
        .search(artist, album, tracks, Duration::seconds(45))
        .await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(e.to_string()))?;
    search.sort_by(|a, b| b.score.total_cmp(&a.score));

    for album in search.iter().take(10) {
        println!("Album: {}", album.album_title);
        println!("Score: {}", album.score);
        println!("Quality: {}", album.dominant_quality);

        for track in album.tracks.iter() {
            println!("  Filename: {:?}", track.base.filename);
            println!("  Title: {:?}", track.title);
            println!("  Artist: {:?}", track.artist);
            println!("  Album: {:?}", track.album);
            println!("  Format: {:?}", track.base.quality());
        }
    }

    Ok(search)
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

#[server]
pub async fn download(data: DownloadQuery) -> Result<Vec<AlbumResult>, ServerFnError> {
    slskd_search(data.album.artist, data.album.title, data.tracks).await
}
