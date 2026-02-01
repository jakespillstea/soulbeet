use dioxus::prelude::*;

pub use crate::models::user_settings::{UpdateUserSettings, UserSettings};

#[cfg(feature = "server")]
use crate::models::app_config::AppConfig;
#[cfg(feature = "server")]
use crate::AuthSession;

#[cfg(feature = "server")]
use super::server_error;

/// Get current user's settings
#[get("/api/settings", auth: AuthSession)]
pub async fn get_user_settings() -> Result<UserSettings, ServerFnError> {
    UserSettings::get(&auth.0.sub).await.map_err(server_error)
}

/// Update current user's settings
#[post("/api/settings", auth: AuthSession)]
pub async fn update_user_settings(update: UpdateUserSettings) -> Result<UserSettings, ServerFnError> {
    UserSettings::upsert(&auth.0.sub, update)
        .await
        .map_err(server_error)
}

/// Get list of available metadata providers
#[get("/api/settings/providers", _: AuthSession)]
pub async fn get_metadata_providers() -> Result<Vec<ProviderInfo>, ServerFnError> {
    use crate::models::app_config::keys;
    use crate::services::{available_metadata_providers, providers};

    let lastfm_available = AppConfig::get(keys::LASTFM_API_KEY)
        .await
        .map_err(server_error)?
        .is_some();

    Ok(available_metadata_providers()
        .into_iter()
        .map(|(id, name)| ProviderInfo {
            id: id.to_string(),
            name: name.to_string(),
            available: match id {
                providers::LASTFM => lastfm_available,
                _ => true,
            },
        })
        .collect())
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub available: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct AppConfigValues {
    pub lastfm_api_key: Option<String>,
    pub slskd_url: Option<String>,
    pub slskd_api_key: Option<String>,
}

#[get("/api/config", _: AuthSession)]
pub async fn get_app_config() -> Result<AppConfigValues, ServerFnError> {
    use crate::models::app_config::keys;

    let lastfm_api_key = AppConfig::get(keys::LASTFM_API_KEY)
        .await
        .map_err(server_error)?;
    let slskd_url = AppConfig::get(keys::SLSKD_URL)
        .await
        .map_err(server_error)?;
    let slskd_api_key = AppConfig::get(keys::SLSKD_API_KEY)
        .await
        .map_err(server_error)?;

    Ok(AppConfigValues {
        lastfm_api_key,
        slskd_url,
        slskd_api_key,
    })
}

#[post("/api/config", _: AuthSession)]
pub async fn update_app_config(config: AppConfigValues) -> Result<AppConfigValues, ServerFnError> {
    use crate::models::app_config::keys;
    use crate::services::reload_providers;

    async fn set_or_delete(key: &str, value: &Option<String>) -> Result<(), ServerFnError> {
        if let Some(v) = value {
            if v.is_empty() {
                AppConfig::delete(key).await.map_err(server_error)?;
            } else {
                AppConfig::set(key, v).await.map_err(server_error)?;
            }
        }
        Ok(())
    }

    set_or_delete(keys::LASTFM_API_KEY, &config.lastfm_api_key).await?;
    set_or_delete(keys::SLSKD_URL, &config.slskd_url).await?;
    set_or_delete(keys::SLSKD_API_KEY, &config.slskd_api_key).await?;

    reload_providers().await;

    get_app_config().await
}
