use std::{collections::HashMap, io::Error as IoError, time::Duration};

use http::StatusCode;
use megamind::{
    models::{
        search::Hit,
        song::RelationshipType,
        Song,
    },
    Client as MegamindClient, ClientError,
};
use moka::future::{Cache, CacheBuilder};
use petgraph::prelude::DiGraphMap;
use serde::{Deserialize, Serialize};
use serde_json::Error as JsonError;
use tokio::{sync::Semaphore, task::JoinError};
use ts_rs::TS;

pub struct AppState {
    pub megamind: MegamindClient,
    pub semaphore: Semaphore,
    pub max_retries: u32,
    song_cache: Cache<u32, Song>,
    search_cache: Cache<String, Vec<Hit>>,
}

impl AppState {
    pub fn new(
        megamind: MegamindClient,
        semaphore_permits: usize,
        max_retries: u32,
    ) -> AppState {
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

    pub async fn song(&self, id: u32) -> Result<Song, ErrIntermediate> {
        let song = self.song_cache.get(&id).await;
        match song {
            Some(result) => Ok(result),
            None => {
                let result = self.megamind.song(id).await?;
                match result {
                    megamind::models::Response::Success { meta: _, response } => {
                        let song = response.song;
                        self.song_cache.insert(id, song.clone()).await;
                        Ok(song)
                    },
                    megamind::models::Response::Error { meta, response } => todo!(),
                    megamind::models::Response::Other { error, error_description } => todo!(),
                }
            },
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Hit>, ErrIntermediate> {
        let hits = self.search_cache.get(query).await;
        match hits {
            Some(result) => Ok(result),
            None => {
                let result = self.megamind.search(query).await?;
                match result {
                    megamind::models::Response::Success { meta: _, response } => {
                        let hits = response.hits;
                        self.search_cache.insert(query.to_string(), hits.clone()).await;
                        Ok(hits)
                    },
                    megamind::models::Response::Error { meta, response } => todo!(),
                    megamind::models::Response::Other { error, error_description } => todo!(),
                }
            }
        }
    }
}

pub struct ErrIntermediate {
    reason: String,
    status: StatusCode,
}

impl ErrIntermediate {
    pub fn new<S: Into<String>>(reason: S, status: StatusCode) -> Self {
        Self {
            reason: reason.into(),
            status,
        }
    }
}

impl From<ClientError> for ErrIntermediate {
    fn from(value: ClientError) -> Self {
        match value {
            ClientError::General(inner) => Self::new(
                inner.to_string(),
                inner.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            ),
            ClientError::RateLimited => {
                Self::new("rate limited by Genius", StatusCode::TOO_MANY_REQUESTS)
            }
        }
    }
}

impl From<JsonError> for ErrIntermediate {
    fn from(value: JsonError) -> Self {
        Self::new(
            format!("serde json error: {}", value),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    }
}

impl From<IoError> for ErrIntermediate {
    fn from(value: IoError) -> Self {
        Self::new(
            format!("IO error: {}", value),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    }
}

impl From<JoinError> for ErrIntermediate {
    fn from(value: JoinError) -> Self {
        Self::new(
            format!("join error: {}", value),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    }
}

impl From<ErrIntermediate> for (String, StatusCode) {
    fn from(value: ErrIntermediate) -> Self {
        (value.reason, value.status)
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
