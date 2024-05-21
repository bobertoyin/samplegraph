use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use futures::stream::{self, StreamExt};
use megamind::models::{RelationshipType, Song};
use petgraph::graphmap::DiGraphMap;

// this value is arbitrary, but we generally want to limit the number of concurrent requests during graph traversal
const MAX_CONCURRENT: usize = 10;

use crate::{error::Error, state::AppState, GraphResponse, SongInfo};

fn is_translation(relationship: RelationshipType) -> bool {
    relationship == RelationshipType::TranslationOf
        || relationship == RelationshipType::Translations
}

pub async fn build_graph(
    state: Arc<AppState>,
    root: u32,
    degrees: u8,
) -> Result<GraphResponse, Error> {
    let mut graph = DiGraphMap::<u32, RelationshipType>::new();
    let mut songs = HashMap::<u32, SongInfo>::new();

    // format for horizon is (song id, song's degree of separation from root)
    let mut horizon = VecDeque::<(u32, u8)>::new();
    horizon.push_back((root, 0));

    while !horizon.is_empty() {
        let chunk = stream::iter(horizon.drain(..))
            .map(|(id, degree)| {
                let state = state.clone();
                async move { (state.song(id).await, degree) }
            })
            .buffer_unordered(MAX_CONCURRENT)
            .collect::<Vec<(Result<Song, Error>, u8)>>()
            .await;

        for (song_result, degree) in chunk {
            let song = song_result?;

            if !graph.contains_node(song.core.essential.id) {
                graph.add_node(song.core.essential.id);
                songs.insert(song.core.essential.id, SongInfo::from((&song, degree)));
            }

            if degree < degrees {
                for relationship in song.song_relationships {
                    if !is_translation(relationship.relationship_type) {
                        for neighbor in relationship.songs {
                            if !graph.contains_node(neighbor.core.essential.id) {
                                graph.add_node(neighbor.core.essential.id);
                                songs.insert(
                                    neighbor.core.essential.id,
                                    SongInfo::from((&neighbor, degree)),
                                );
                                horizon.push_back((neighbor.core.essential.id, degree + 1));
                            }

                            if !graph
                                .contains_edge(song.core.essential.id, neighbor.core.essential.id)
                                && !graph.contains_edge(
                                    neighbor.core.essential.id,
                                    song.core.essential.id,
                                )
                            {
                                graph.add_edge(
                                    song.core.essential.id,
                                    neighbor.core.essential.id,
                                    relationship.relationship_type,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(GraphResponse { graph, songs })
}
