#[cfg(feature = "server")]
use crate::db::DB;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct UserSettings {
    pub user_id: String,
    pub default_metadata_provider: Option<String>,
    pub last_search_type: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateUserSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_metadata_provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_search_type: Option<String>,
}

#[cfg(feature = "server")]
impl UserSettings {
    pub async fn get(user_id: &str) -> Result<UserSettings, String> {
        // Try to get existing settings, or return defaults
        let settings = sqlx::query_as::<_, UserSettings>(
            "SELECT * FROM user_settings WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_optional(&*DB)
        .await
        .map_err(|e| e.to_string())?;

        Ok(settings.unwrap_or_else(|| UserSettings {
            user_id: user_id.to_string(),
            default_metadata_provider: Some("musicbrainz".to_string()),
            last_search_type: Some("album".to_string()),
        }))
    }

    pub async fn upsert(user_id: &str, update: UpdateUserSettings) -> Result<UserSettings, String> {
        // Build dynamic update - only update fields that are Some
        let current = Self::get(user_id).await?;

        let provider = update.default_metadata_provider.or(current.default_metadata_provider);
        let search_type = update.last_search_type.or(current.last_search_type);

        sqlx::query(
            r#"
            INSERT INTO user_settings (user_id, default_metadata_provider, last_search_type)
            VALUES (?, ?, ?)
            ON CONFLICT(user_id) DO UPDATE SET
                default_metadata_provider = excluded.default_metadata_provider,
                last_search_type = excluded.last_search_type
            "#,
        )
        .bind(user_id)
        .bind(&provider)
        .bind(&search_type)
        .execute(&*DB)
        .await
        .map_err(|e| e.to_string())?;

        Self::get(user_id).await
    }
}
