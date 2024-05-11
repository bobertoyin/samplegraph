use std::time::Duration;

use megamind::{
    models::{search::Hit, Song},
    Client as MegamindClient,
};
use moka::future::{Cache, CacheBuilder};
use serde::Deserialize;

use crate::error::Error;

pub struct AppState {
    pub megamind: MegamindClient,
    song_cache: Cache<u32, Song>,
    search_cache: Cache<String, Vec<Hit>>,
}

impl AppState {
    pub fn new(megamind: MegamindClient) -> AppState {
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
            song_cache,
            search_cache,
        }
    }

    pub async fn song(&self, id: u32) -> Result<Song, Error> {
        let song = self.song_cache.get(&id).await;
        match song {
            Some(result) => Ok(result),
            None => {
                let song = Error::from_genius_response(self.megamind.song(id).await?)?.song;
                self.song_cache.insert(id, song.clone()).await;
                Ok(song)
            }
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Hit>, Error> {
        let hits = self.search_cache.get(query).await;
        match hits {
            Some(result) => Ok(result),
            None => {
                let hits = Error::from_genius_response(self.megamind.search(query).await?)?.hits;
                self.search_cache
                    .insert(query.to_string(), hits.clone())
                    .await;
                Ok(hits)
            }
        }
    }
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub query: String,
}
