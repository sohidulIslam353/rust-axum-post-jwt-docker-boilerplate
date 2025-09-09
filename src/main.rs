use axum::Extension;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
mod app;
mod config;
mod models;
mod routes;

use config::{database, redis};
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv().ok();
    config::jwt::init_keys().expect("Failed to initialize JWT keys");

    // Connect to database
    let db = database::connect_db()
        .await
        .expect("Failed to connect to database");

    // Initialize KeyDB/Redis cache
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    let cache = redis::init_cache(&redis_url)
        .await
        .expect("Failed to connect to cache");

    // Set up tracing for better logging and debugging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Create main app router
    let app = routes::create_routes()
        .layer(Extension(db))
        .layer(Extension(Arc::new(Mutex::new(cache))));

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("Failed to parse port");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!("Listening on {}", addr);

    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
