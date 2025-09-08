use std::net::SocketAddr;

mod config;
use config::{cache, database};
mod app;
mod routes;

use dotenvy::dotenv;
#[tokio::main]
async fn main() {
    dotenv().ok(); // Load environment variables from .env without docker-compose
    // Connect to database
    let __db = database::connect_db().await;

    // Initialize KeyDB/Redis for dockerc-compose
    //   let _cache = cache::init_cache("redis://keydb:6379/").await;

    // without docker
    let _cache = cache::init_cache("redis://127.0.0.1:6379/").await;

    // Create main app router
    let app = routes::create_routes();

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
