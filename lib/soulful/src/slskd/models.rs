use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub username: String,
    pub filename: String,
    pub size: i64,
    pub bitrate: Option<i32>,
    pub duration: Option<i32>,
    pub has_free_upload_slot: bool,
    pub upload_speed: i32,
    pub queue_length: i32,
}

impl SearchResult {
    pub fn quality(&self) -> String {
        Path::new(&self.filename)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_lowercase()
    }

    pub fn quality_score(&self) -> f64 {
        let quality_weights: HashMap<&str, f64> = [
            ("flac", 1.0),
            ("mp3", 0.8),
            ("ogg", 0.7),
            ("aac", 0.6),
            ("wma", 0.5),
        ]
        .iter()
        .cloned()
        .collect();

        let mut base_score = *quality_weights.get(self.quality().as_str()).unwrap_or(&0.3);

        if let Some(br) = self.bitrate {
            if br >= 320 {
                base_score += 0.2;
            } else if br >= 256 {
                base_score += 0.1;
            } else if br < 128 {
                base_score -= 0.2;
            }
        }

        if self.has_free_upload_slot {
            base_score += 0.1;
        }
        if self.upload_speed > 100 {
            base_score += 0.05;
        }
        if self.queue_length > 10 {
            base_score -= 0.1;
        }

        base_score.min(1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackResult {
    #[serde(flatten)]
    pub base: SearchResult,
    pub artist: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
    pub track_number: Option<i32>,
}

impl TrackResult {
    pub fn new(base: SearchResult) -> Self {
        let mut track = Self {
            base,
            artist: None,
            title: None,
            album: None,
            track_number: None,
        };
        track.parse_filename_metadata();
        track
    }

    fn parse_filename_metadata(&mut self) {
        let file_stem = Path::new(&self.base.filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let patterns = [
            // "01 - Artist - Title" or "01. Artist - Title"
            r"^(?P<track>\d+)\s*[-\.]\s*(?P<artist>.+?)\s*[-–]\s*(?P<title>.+)$",
            // "Artist - Title"
            r"^(?P<artist>.+?)\s*[-–]\s*(?P<title>.+)$",
            // "01 - Title" or "01. Title"
            r"^(?P<track>\d+)\s*[-\.]\s*(?P<title>.+)$",
        ];

        for pattern in patterns.iter() {
            let re = Regex::new(pattern).unwrap();
            if let Some(caps) = re.captures(file_stem) {
                if let Some(track) = caps.name("track") {
                    self.track_number = track.as_str().parse().ok();
                }
                if let Some(artist) = caps.name("artist") {
                    self.artist = Some(artist.as_str().trim().to_string());
                }
                if let Some(title) = caps.name("title") {
                    self.title = Some(title.as_str().trim().to_string());
                }
                break;
            }
        }

        if self.title.is_none() {
            self.title = Some(file_stem.to_string());
        }

        if self.album.is_none() {
            let path_parts: Vec<&str> = self
                .base
                .filename
                .split(|c| c == '/' || c == '\\')
                .collect();
            if path_parts.len() >= 2 {
                if let Some(album_part) = path_parts.iter().rev().nth(1) {
                    if !album_part.starts_with('@') && album_part.len() > 3 {
                        self.album = Some(album_part.to_string());
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumResult {
    pub username: String,
    pub album_path: String,
    pub album_title: String,
    pub artist: Option<String>,
    pub track_count: usize,
    pub total_size: i64,
    pub tracks: Vec<TrackResult>,
    pub dominant_quality: String,
    pub year: Option<String>,
    pub has_free_upload_slot: bool,
    pub upload_speed: i32,
    pub queue_length: i32,
}

impl AlbumResult {
    pub fn quality_score(&self) -> f64 {
        let quality_weights: HashMap<&str, f64> = [
            ("flac", 1.0),
            ("mp3", 0.8),
            ("ogg", 0.7),
            ("aac", 0.6),
            ("wma", 0.5),
        ]
        .iter()
        .cloned()
        .collect();

        let mut base_score = *quality_weights
            .get(self.dominant_quality.as_str())
            .unwrap_or(&0.3);

        if (8..=20).contains(&self.track_count) {
            base_score += 0.1;
        } else if self.track_count > 20 {
            base_score += 0.05;
        }

        if self.has_free_upload_slot {
            base_score += 0.1;
        }
        if self.upload_speed > 100 {
            base_score += 0.05;
        }
        if self.queue_length > 10 {
            base_score -= 0.1;
        }

        base_score.min(1.0)
    }

    pub fn size_mb(&self) -> i64 {
        self.total_size / (1024 * 1024)
    }

    pub fn average_track_size_mb(&self) -> f64 {
        if self.track_count > 0 {
            self.size_mb() as f64 / self.track_count as f64
        } else {
            0.0
        }
    }
}

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
