use std::{collections::HashMap, time::Duration};

use megamind::{
    models::{search::Hit, song::RelationshipType, Song},
    Client as MegamindClient,
};
use moka::future::{Cache, CacheBuilder};
use petgraph::prelude::DiGraphMap;
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use ts_rs::TS;

use crate::error::Error;

pub struct AppState {
    pub megamind: MegamindClient,
    pub semaphore: Semaphore,
    pub max_retries: u32,
    song_cache: Cache<u32, Song>,
    search_cache: Cache<String, Vec<Hit>>,
}

impl AppState {
    pub fn new(megamind: MegamindClient, semaphore_permits: usize, max_retries: u32) -> AppState {
        let semaphore = Semaphore::new(semaphore_permits);
        let song_cache = CacheBuilder::default()
            .time_to_live(Duration::from_secs(10 * 60))
            .max_capacity(10_000)
            .build();
        let search_cache = CacheBuilder::default()
            .time_to_live(Duration::from_secs(30))
            .max_capacity(1_000)
            .build();
        Self {
            megamind,
            semaphore,
            max_retries,
            song_cache,
            search_cache,
        }
    }

    pub async fn song(&self, id: u32) -> Result<Song, Error> {
        let song = self.song_cache.get(&id).await;
        match song {
            Some(result) => Ok(result),
            None => {
                let result = self.megamind.song(id).await?;
                Error::from_genius_response(result).map(|response| response.song)
            }
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Hit>, Error> {
        let hits = self.search_cache.get(query).await;
        match hits {
            Some(result) => Ok(result),
            None => {
                let result = self.megamind.search(query).await?;
                Error::from_genius_response(result).map(|response| response.hits)
            }
        }
    }
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
#[ts(export, export_to = "./client/src/bindings/GraphResponse.d.ts")]
pub struct GraphResponse {
    #[ts(type = "{ nodes: Array<number>, edges: Array<[number, number, string]> }")]
    pub graph: DiGraphMap<u32, RelationshipType>,
    pub songs: HashMap<String, SongInfo>,
}

#[derive(Serialize, TS)]
#[ts(export, export_to = "./client/src/bindings/SearchResponse.d.ts")]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
}

impl From<Vec<Hit>> for SearchResponse {
    fn from(value: Vec<Hit>) -> Self {
        Self {
            hits: value
                .into_iter()
                .map(|hit| match hit {
                    Hit::Song(song) => SearchHit {
                        full_title: song.result.core.full_title,
                        id: song.result.core.essential.id,
                    },
                })
                .collect(),
        }
    }
}

#[derive(Serialize, TS)]
#[ts(export, export_to = "./client/src/bindings/SearchHit.d.ts")]
pub struct SearchHit {
    pub full_title: String,
    pub id: u32,
}

#[derive(Deserialize)]
pub struct GraphQuery {
    pub degree: Option<u8>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub query: String,
}
