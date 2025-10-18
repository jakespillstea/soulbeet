use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadStatus {
    pub id: String,
    pub filename: String,
    pub username: String,
    pub state: String,
    #[serde(rename = "percentComplete")]
    pub progress: f32,
    pub size: i64,
    #[serde(rename = "bytesTransferred")]
    pub transferred: i64,
    #[serde(rename = "averageSpeed")]
    pub speed: i32,
    pub time_remaining: Option<i32>,
}

// Internal structs for deserializing raw API responses
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchResponseFile {
    pub filename: String,
    pub size: i64,
    pub bit_rate: Option<i32>,
    pub length: Option<i32>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchResponse {
    pub username: String,
    pub files: Vec<SearchResponseFile>,
    pub has_free_upload_slot: bool,
    pub upload_speed: i32,
    pub queue_length: i32,
}

#[derive(Serialize)]
pub(crate) struct DownloadRequestFile<'a> {
    pub filename: &'a str,
    pub size: i64,
    pub path: String,
}
