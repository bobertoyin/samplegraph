use std::env::var;

use actix_web::{
    get,
    middleware::{Compress as CompressMW, Logger},
    web::{scope, Data, Json, Path, Query},
    App, HttpServer, Responder,
};
use actix_web_lab::web::spa;
use env_logger::init as init_logger;
use megamind::ClientBuilder;

use samplegraph::{
    error::Error,
    graph::build_graph,
    state::{AppState, SearchQuery},
    types::*,
};

#[get("/version")]
async fn get_version() -> impl Responder {
    Json(env!("CARGO_PKG_VERSION"))
}

#[get("/graph/{song_id}")]
async fn get_graph(path: Path<u32>, data: Data<AppState>) -> Result<Json<GraphResponse>, Error> {
    let root = path.into_inner();
    match build_graph(data.into_inner(), root, 2).await {
        Ok(graph) => Ok(Json(graph)),
        Err(error) => Err(error),
    }
}

#[get("/search")]
async fn get_search(
    query: Query<SearchQuery>,
    data: Data<AppState>,
) -> Result<Json<SearchResponse>, Error> {
    match data.search(query.query.as_ref()).await {
        Ok(hits) => Ok(Json(hits.into())),
        Err(error) => Err(error),
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
        let state = AppState::new(megamind);
        let api_service = scope("/api")
            .service(get_version)
            .service(get_graph)
            .service(get_search);
        let spa_service = spa()
            .index_file("./frontend/dist/index.html")
            .static_resources_mount("/")
            .static_resources_location("./frontend/dist")
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
