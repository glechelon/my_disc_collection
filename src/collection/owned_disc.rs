use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct OwnedDisc {
    pub id: Option<i32>,
    pub title: String,
    pub cover: String,
    pub artist: OwnedArtist,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OwnedArtist {
    pub id: Option<i32>,
    pub name: String,
    pub picture: String,
}
