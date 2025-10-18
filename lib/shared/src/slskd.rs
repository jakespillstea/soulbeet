use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct MatchResult {
    pub guessed_artist: String,
    pub guessed_album: String,
    pub matched_track: String,
    pub artist_score: f64,
    pub album_score: f64,
    pub track_score: f64,
    pub total_score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackResult {
    #[serde(flatten)]
    pub base: SearchResult,
    pub artist: String,
    pub title: String,
    pub album: String,
    pub match_score: f64,
}

impl TrackResult {
    pub fn new(base: SearchResult, matched: MatchResult) -> Self {
        Self {
            base,
            artist: matched.guessed_artist,
            title: matched.matched_track,
            album: matched.guessed_album,
            match_score: matched.total_score,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
            ("wav", 0.95),
            ("m4a", 0.65),
            ("aac", 0.65),
            ("mp3", 0.55),
            ("ogg", 0.6),
            ("wma", 0.4),
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
                base_score -= 0.3;
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlbumResult {
    pub username: String,
    pub album_path: String,
    pub album_title: String,
    pub artist: Option<String>,
    pub track_count: usize,
    pub total_size: i64,
    pub tracks: Vec<TrackResult>,
    pub dominant_quality: String,
    pub has_free_upload_slot: bool,
    pub upload_speed: i32,
    pub queue_length: i32,
    pub score: f64,
}

impl AlbumResult {
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
