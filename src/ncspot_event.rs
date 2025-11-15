use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct NcspotEvent {
    pub mode: Mode,
    pub playable: Option<Playable>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Mode {
    Playing {
        #[serde(rename = "Playing")]
        playing: PlayingTimestamp,
    },
    Paused {
        #[serde(rename = "Paused")]
        paused: PausedPosition,
    },
    Simple(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayingTimestamp {
    pub secs_since_epoch: u64,
    pub nanos_since_epoch: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PausedPosition {
    pub secs: u64,
    pub nanos: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Playable {
    #[serde(rename = "type")]
    pub playable_type: String,
    pub id: String,
    pub uri: String,
    pub title: String,
    pub track_number: u32,
    pub disc_number: u32,
    pub duration: u64,
    pub artists: Vec<String>,
    pub artist_ids: Vec<String>,
    pub album: String,
    pub album_id: String,
    pub album_artists: Vec<String>,
    pub cover_url: String,
    pub url: String,
    pub added_at: Option<String>,
    pub list_index: u32,
    pub is_local: bool,
    pub is_playable: bool,
}
