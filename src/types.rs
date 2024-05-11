use std::collections::HashMap;

use megamind::models::{search::Hit, song::RelationshipType};
use petgraph::prelude::DiGraphMap;
use serde::Serialize;
use ts_rs::TS;

#[derive(Serialize, TS)]
#[ts(export, export_to = "./client/src/bindings/GraphResponse.d.ts")]
pub struct GraphResponse {
    #[ts(type = "{ nodes: Array<number>, edges: Array<[number, number, string]> }")]
    pub graph: DiGraphMap<u32, RelationshipType>,
    pub songs: HashMap<u32, SongInfo>,
}

#[derive(Serialize, TS)]
#[ts(export, export_to = "./client/src/bindings/SongInfo.d.ts")]
pub struct SongInfo {
    pub full_title: String,
    pub url: String,
    pub thumbnail: String,
    pub degree: u8,
}

#[derive(Serialize, TS)]
#[ts(export, export_to = "./client/src/bindings/SearchResponse.d.ts")]
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
#[ts(export, export_to = "./client/src/bindings/SearchHit.d.ts")]
pub struct SearchHit {
    pub full_title: String,
    pub id: u32,
}

impl From<Hit> for SearchHit {
    fn from(value: Hit) -> Self {
        match value {
            Hit::Song(song) => Self {
                full_title: song.result.core.full_title,
                id: song.result.core.essential.id,
            },
        }
    }
}
