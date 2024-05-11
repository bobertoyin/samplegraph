use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use futures::{stream, StreamExt, TryStreamExt};
use megamind::models::{RelationshipType, Song};
use petgraph::graphmap::DiGraphMap;
use tokio::spawn;

// this value is arbitrary, but we generally want to limit the number of tasks spawned during graph traversal
const MAX_CONCURRENT: usize = 100;

use crate::{error::Error, state::AppState, GraphResponse, SongInfo};

fn invert(relationship: RelationshipType) -> RelationshipType {
    match relationship {
        RelationshipType::Samples => RelationshipType::SampledIn,
        RelationshipType::SampledIn => RelationshipType::Samples,
        RelationshipType::Interpolates => RelationshipType::InterpolatedBy,
        RelationshipType::InterpolatedBy => RelationshipType::Interpolates,
        RelationshipType::CoverOf => RelationshipType::CoveredBy,
        RelationshipType::CoveredBy => RelationshipType::CoverOf,
        RelationshipType::RemixOf => RelationshipType::RemixedBy,
        RelationshipType::RemixedBy => RelationshipType::RemixOf,
        RelationshipType::LiveVersionOf => RelationshipType::PerformedLiveAs,
        RelationshipType::PerformedLiveAs => RelationshipType::LiveVersionOf,
        RelationshipType::TranslationOf => RelationshipType::Translations,
        RelationshipType::Translations => RelationshipType::TranslationOf,
        RelationshipType::Unknown => RelationshipType::Unknown,
    }
}

fn is_translation(relationship: RelationshipType) -> bool {
    relationship == RelationshipType::TranslationOf || relationship == RelationshipType::Translations
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
        let horizon_search = stream::iter(
            horizon
                .drain(..)
                .filter(|(_, degree)| *degree <= degrees)
                .map(|(id, degree)| {
                    let state = state.clone();
                    spawn(async move { (state.song(id).await, degree) })
                }),
        )
        .buffer_unordered(MAX_CONCURRENT)
        .try_collect::<Vec<(Result<Song, Error>, u8)>>()
        .await?;

        for (song_result, degree) in horizon_search {
            let song = song_result?;

            if !graph.contains_node(song.core.essential.id) {
                graph.add_node(song.core.essential.id);
                songs.insert(
                    song.core.essential.id,
                    SongInfo {
                        full_title: song.core.full_title,
                        url: song.core.essential.url,
                        degree,
                        thumbnail: song.core.song_art_image_thumbnail_url,
                    },
                );
            }

            if degree < degrees {
                for relationship in song.song_relationships {
                    if !is_translation(relationship.relationship_type) {
                        for neighbor in relationship.songs {

                            if !graph.contains_node(neighbor.core.essential.id) {
                                graph.add_node(neighbor.core.essential.id);
                                songs.insert(
                                    neighbor.core.essential.id,
                                    SongInfo {
                                        full_title: neighbor.core.full_title,
                                        url: neighbor.core.essential.url,
                                        degree,
                                        thumbnail: neighbor.core.song_art_image_thumbnail_url,
                                    },
                                );
                                horizon.push_back((neighbor.core.essential.id, degree + 1));
                            }

                            if !graph.contains_edge(song.core.essential.id, neighbor.core.essential.id) {
                                graph.add_edge(song.core.essential.id, neighbor.core.essential.id, relationship.relationship_type);
                            }

                            if !graph.contains_edge(neighbor.core.essential.id, song.core.essential.id) {
                                graph.add_edge(neighbor.core.essential.id, song.core.essential.id, invert(relationship.relationship_type));
                            }

                        }
                    }
                }
            }
        }
    }
    
    Ok(GraphResponse { graph, songs })
}
