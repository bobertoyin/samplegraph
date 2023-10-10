use std::{
    cmp::min,
    collections::{HashMap, VecDeque},
    env::var,
    sync::Arc,
};

use actix_web::{
    get,
    middleware::{Compress as CompressMW, Logger},
    web::{scope, Data, Json, Path, Query},
    App, Either, HttpServer, Responder,
};
use actix_web_lab::web::spa;
use env_logger::init as init_logger;
use futures::future::join_all;
use http::StatusCode;
use log::info;
use megamind::{models::RelationshipType, ClientBuilder};
use petgraph::prelude::DiGraphMap;
use redis::Client as RedisClient;
use tokio::{
    sync::Semaphore,
    task::spawn,
    time::{sleep, Duration},
};

use server::*;

async fn build_graph(
    state: Arc<AppState>,
    start_id: u32,
    max_degree: u8,
) -> Result<GraphResponse, (String, StatusCode)> {
    let max_tasks: usize = 128;
    let mut retries = 0;
    let mut graph = DiGraphMap::new();
    let mut songs = HashMap::new();
    let mut search_queue = VecDeque::new();

    search_queue.push_back((start_id, 1));

    let state_ref = state.as_ref();

    while !search_queue.is_empty() && retries < state_ref.max_retries {
        let permit = state.semaphore.try_acquire_many(max_tasks as u32);
        if permit.is_ok() {
            let amount = min(search_queue.len(), max_tasks);
            let next_tasks = search_queue
                .drain(0..amount)
                .filter(|item| item.1 <= max_degree);

            let completed_tasks = join_all(next_tasks.into_iter().map(|(id, degree)| {
                let state_clone = state.clone();
                spawn(async move { (state_clone.song(id).await, degree) })
            }))
            .await;

            for task_result in completed_tasks {
                let (result, degree) = task_result.map_err(ErrIntermediate::from)?;
                let response = result?;
                if !graph.contains_node(response.song.core.essential.id) {
                    graph.add_node(response.song.core.essential.id);
                    songs.insert(
                        response.song.core.essential.id.to_string(),
                        SongInfo {
                            full_title: response.song.core.full_title,
                            url: response.song.core.essential.url,
                            degree,
                            thumbnail: response.song.core.song_art_image_thumbnail_url,
                        },
                    );
                }

                if degree < max_degree {
                    for relationship in response.song.song_relationships {
                        if relationship.relationship_type != RelationshipType::TranslationOf
                            && relationship.relationship_type != RelationshipType::Translations
                        {
                            for song in relationship.songs {
                                if !graph.contains_node(song.core.essential.id) {
                                    search_queue.push_back((song.core.essential.id, degree + 1));
                                    graph.add_node(response.song.core.essential.id);
                                    songs.insert(
                                        song.core.essential.id.to_string(),
                                        SongInfo {
                                            full_title: song.core.full_title,
                                            url: song.core.essential.url,
                                            degree: degree + 1,
                                            thumbnail: song.core.song_art_image_thumbnail_url,
                                        },
                                    );
                                    if !graph.contains_edge(
                                        response.song.core.essential.id,
                                        song.core.essential.id,
                                    ) {
                                        graph.add_edge(
                                            response.song.core.essential.id,
                                            song.core.essential.id,
                                            relationship.relationship_type,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            sleep(Duration::from_secs(2)).await;
            info!("failed to get semaphore permit, waiting to retry...");
            retries += 1;
        }
    }

    if retries >= state.max_retries {
        return Err((
            String::from("server resources exhausted"),
            StatusCode::SERVICE_UNAVAILABLE,
        ));
    }

    Ok(GraphResponse { graph, songs })
}

#[get("/version")]
async fn get_version() -> impl Responder {
    Json(env!("CARGO_PKG_VERSION"))
}

#[get("/graph/{song_id}")]
async fn get_graph(
    path: Path<u32>,
    query: Query<GraphQuery>,
    data: Data<AppState>,
) -> Either<(Json<GraphResponse>, StatusCode), (String, StatusCode)> {
    let song_id = path.into_inner();
    let max_degree = query.degree.unwrap_or(3);
    match build_graph(data.into_inner(), song_id, max_degree).await {
        Ok(graph) => Either::Left((Json(graph), StatusCode::OK)),
        Err(error) => Either::Right(error),
    }
}

#[get("/search")]
async fn get_search(
    query: Query<SearchQuery>,
    data: Data<AppState>,
) -> Either<(Json<SearchResponse>, StatusCode), (String, StatusCode)> {
    match data.search(query.query.as_ref()).await {
        Ok(hits) => Either::Left((Json(hits.into()), StatusCode::OK)),
        Err(error) => Either::Right(error.into()),
    }
}

#[tokio::main]
async fn main() {
    init_logger();

    HttpServer::new(move || {
        let megamind = ClientBuilder::new()
            .auth_token(var("GENIUS_TOKEN").expect("missing Genius token"))
            .build()
            .expect("failed to create Genius client");
        let redis = RedisClient::open(var("REDIS_URL").expect("missing Redis URL"))
            .expect("failed to create Redis client");
        let semaphore = Semaphore::new(4096);
        let state = AppState {
            megamind,
            redis,
            semaphore,
            max_retries: 10,
        };
        let api_service = scope("/api")
            .service(get_version)
            .service(get_graph)
            .service(get_search);
        let spa_service = spa()
            .index_file("../client/build/index.html")
            .static_resources_mount("/")
            .static_resources_location("../client/build")
            .finish();
        App::new()
            .app_data(Data::new(state))
            .wrap(CompressMW::default())
            .wrap(Logger::default())
            .service(api_service)
            .service(spa_service)
    })
    .bind(("0.0.0.0", 8080))
    .expect("failed to bind server to address")
    .run()
    .await
    .expect("failed to run server")
}
