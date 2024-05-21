use std::collections::HashMap;

use megamind::models::{search::Hit, song::RelationshipType, Song, SongCoreStats, SongCoreWithRDC};
use petgraph::prelude::DiGraphMap;
use serde::Serialize;
use ts_rs::TS;

#[derive(Serialize, TS)]
#[ts(export, export_to = "./frontend/src/bindings/GraphResponse.d.ts")]
pub struct GraphResponse {
    #[ts(type = "{ nodes: Array<number>, edges: Array<[number, number, string]> }")]
    pub graph: DiGraphMap<u32, RelationshipType>,
    pub songs: HashMap<u32, SongInfo>,
}

#[derive(Serialize, TS)]
#[ts(export, export_to = "./frontend/src/bindings/SongInfo.d.ts")]
pub struct SongInfo {
    pub title: String,
    pub artist: String,
    pub url: String,
    pub thumbnail: String,
    pub degree: u8,
}

impl From<(&Song, u8)> for SongInfo {
    fn from(value: (&Song, u8)) -> Self {
        let (song, degree) = value;
        Self {
            title: song.core.essential.title.clone(),
            artist: song.core.primary_artist.name.clone(),
            url: song.core.essential.url.clone(),
            degree,
            thumbnail: song.core.song_art_image_thumbnail_url.clone(),
        }
    }
}

impl From<(&SongCoreWithRDC<SongCoreStats>, u8)> for SongInfo {
    fn from(value: (&SongCoreWithRDC<SongCoreStats>, u8)) -> Self {
        let (song, degree) = value;
        Self {
            title: song.core.essential.title.clone(),
            artist: song.core.primary_artist.name.clone(),
            url: song.core.essential.url.clone(),
            degree,
            thumbnail: song.core.song_art_image_thumbnail_url.clone(),
        }
    }
}

#[derive(Serialize, TS)]
#[ts(export, export_to = "./frontend/src/bindings/SearchResponse.d.ts")]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
}

impl From<Vec<Hit>> for SearchResponse {
    fn from(value: Vec<Hit>) -> Self {
        Self {
            hits: value.into_iter().map(SearchHit::from).collect(),
        }
    }
}

#[derive(Serialize, TS)]
#[ts(export, export_to = "./frontend/src/bindings/SearchHit.d.ts")]
pub struct SearchHit {
    pub title: String,
    pub artist: String,
    pub id: u32,
    pub thumbnail: String,
    pub url: String,
}

impl From<Hit> for SearchHit {
    fn from(value: Hit) -> Self {
        match value {
            Hit::Song(song) => Self {
                title: song.result.core.essential.title,
                artist: song.result.core.primary_artist.name,
                id: song.result.core.essential.id,
                thumbnail: song.result.core.song_art_image_thumbnail_url,
                url: song.result.core.essential.url,
            },
        }
    }
}
