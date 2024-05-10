use std::{collections::HashMap, io::Error as IoError};

use http::StatusCode;
use megamind::{
    models::{
        search::{Hit, SearchResponse as MegamindSearchResponse},
        song::RelationshipType,
    },
    Client as MegamindClient, ClientError,
};
use petgraph::prelude::DiGraphMap;
use redis::{Client as RedisClient, RedisError};
use serde::{Deserialize, Serialize};
use serde_json::Error as JsonError;
use tokio::{sync::Semaphore, task::JoinError};
use ts_rs::TS;

pub struct AppState {
    pub megamind: MegamindClient,
    pub redis: RedisClient,
    pub semaphore: Semaphore,
    pub max_retries: u32,
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

impl From<RedisError> for ErrIntermediate {
    fn from(value: RedisError) -> Self {
        Self::new(
            format!("redis error: {}", value),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
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

impl From<MegamindSearchResponse> for SearchResponse {
    fn from(value: MegamindSearchResponse) -> Self {
        Self {
            hits: value
                .hits
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
