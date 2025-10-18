use serde::{Deserialize, Serialize};

use crate::musicbrainz::{Album, Track};

#[derive(Serialize, Clone, PartialEq, Deserialize, Debug)]
pub struct DownloadQuery {
    pub album: Album,
    pub tracks: Vec<Track>,
}
