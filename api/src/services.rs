#[cfg(feature = "server")]
use std::{collections::HashMap, sync::{Arc, LazyLock}};

#[cfg(feature = "server")]
use soulbeet::{
    musicbrainz::MusicBrainzProvider,
    slskd::{DownloadConfig, SoulseekClientBuilder},
    DownloadBackend, LastFmProvider, MetadataProvider,
};
#[cfg(feature = "server")]
use tokio::sync::RwLock;

#[cfg(feature = "server")]
use crate::models::app_config::{keys, AppConfig};

pub mod providers {
    pub const MUSICBRAINZ: &str = "musicbrainz";
    pub const LASTFM: &str = "lastfm";
}

pub mod downloaders {
    pub const SLSKD: &str = "slskd";
}

#[cfg(feature = "server")]
static METADATA_PROVIDERS: LazyLock<RwLock<HashMap<String, Arc<dyn MetadataProvider>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

#[cfg(feature = "server")]
static DOWNLOAD_BACKENDS: LazyLock<RwLock<HashMap<String, Arc<dyn DownloadBackend>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

#[cfg(feature = "server")]
pub fn available_metadata_providers() -> Vec<(&'static str, &'static str)> {
    vec![
        (providers::MUSICBRAINZ, "MusicBrainz"),
        (providers::LASTFM, "Last.fm"),
    ]
}

#[cfg(feature = "server")]
pub fn available_download_backends() -> Vec<(&'static str, &'static str)> {
    vec![(downloaders::SLSKD, "Soulseek")]
}

#[cfg(feature = "server")]
async fn init_metadata_provider(id: &str) -> Result<Arc<dyn MetadataProvider>, String> {
    match id {
        providers::LASTFM => {
            let api_key = AppConfig::get(keys::LASTFM_API_KEY)
                .await?
                .ok_or("Last.fm API key not configured")?;
            if api_key.is_empty() {
                return Err("Last.fm API key not configured".to_string());
            }
            Ok(Arc::new(LastFmProvider::new(api_key)))
        }
        _ => Ok(Arc::new(MusicBrainzProvider::new())),
    }
}

#[cfg(feature = "server")]
async fn init_download_backend(id: &str) -> Result<Arc<dyn DownloadBackend>, String> {
    match id {
        downloaders::SLSKD => {
            let url = AppConfig::get(keys::SLSKD_URL)
                .await?
                .ok_or("slskd URL not configured")?;
            let api_key = AppConfig::get(keys::SLSKD_API_KEY)
                .await?
                .ok_or("slskd API key not configured")?;

            if url.is_empty() || api_key.is_empty() {
                return Err("slskd not configured".to_string());
            }

            let client = SoulseekClientBuilder::new()
                .base_url(&url)
                .api_key(&api_key)
                .download_config(DownloadConfig {
                    batch_size: 3,
                    batch_delay_ms: 3000,
                    max_retries: 3,
                    retry_base_delay_ms: 1000,
                })
                .build()
                .map_err(|e| e.to_string())?;

            Ok(Arc::new(client))
        }
        _ => Err(format!("Unknown download backend: {}", id)),
    }
}

#[cfg(feature = "server")]
pub async fn metadata_provider(id: Option<&str>) -> Result<Arc<dyn MetadataProvider>, String> {
    let requested = id.unwrap_or(providers::MUSICBRAINZ);

    if let Some(provider) = METADATA_PROVIDERS.read().await.get(requested) {
        return Ok(provider.clone());
    }

    let (key, provider) = match init_metadata_provider(requested).await {
        Ok(p) => (requested.to_string(), p),
        Err(_) if requested != providers::MUSICBRAINZ => {
            (providers::MUSICBRAINZ.to_string(), init_metadata_provider(providers::MUSICBRAINZ).await?)
        }
        Err(e) => return Err(e),
    };

    METADATA_PROVIDERS.write().await.insert(key, provider.clone());
    Ok(provider)
}

#[cfg(feature = "server")]
pub async fn download_backend(id: Option<&str>) -> Result<Arc<dyn DownloadBackend>, String> {
    let requested = id.unwrap_or(downloaders::SLSKD);

    if let Some(backend) = DOWNLOAD_BACKENDS.read().await.get(requested) {
        return Ok(backend.clone());
    }

    let backend = init_download_backend(requested).await?;
    DOWNLOAD_BACKENDS.write().await.insert(requested.to_string(), backend.clone());
    Ok(backend)
}

#[cfg(feature = "server")]
pub async fn reload_providers() {
    METADATA_PROVIDERS.write().await.clear();
    DOWNLOAD_BACKENDS.write().await.clear();
}

#[cfg(feature = "server")]
pub async fn is_slskd_configured() -> bool {
    let url = AppConfig::get(keys::SLSKD_URL).await.ok().flatten();
    let api_key = AppConfig::get(keys::SLSKD_API_KEY).await.ok().flatten();
    matches!((url, api_key), (Some(u), Some(k)) if !u.is_empty() && !k.is_empty())
}
