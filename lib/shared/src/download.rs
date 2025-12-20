use serde::{Deserialize, Serialize};

use crate::musicbrainz::{Album, Track};

#[derive(Serialize, Clone, PartialEq, Deserialize, Debug)]
pub struct DownloadQuery {
    pub album: Option<Album>,
    pub tracks: Vec<Track>,
}

impl From<Track> for DownloadQuery {
    fn from(track: Track) -> Self {
        DownloadQuery {
            album: None,
            tracks: vec![track],
        }
    }
}
