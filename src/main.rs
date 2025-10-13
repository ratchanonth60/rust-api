// src/main.rs

use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Declare modules
mod config;
mod db;
mod errors;
mod handlers;
mod models;
mod routes;
mod schema;
mod security;
mod state;

#[tokio::main]
async fn main() {
    // Load configuration from .env file
    dotenvy::dotenv().ok();
    let config = config::AppConfig::from_env();

    // Setup logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "api=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create database connection pool
    let db_pool = db::connect::establish_connection(&config.database_url);

    // Create application state
    let app_state = state::AppState {
        db_pool,
        config: config.clone(),
    };

    // Create the router
    let app = routes::create_router(app_state);

    // Run the server
    let addr = SocketAddr::from((
        config.server_host.parse::<std::net::IpAddr>().unwrap(),
        config.server_port,
    ));

    info!("Server listening on http://{}", addr);
    info!("Swagger UI available at http://{}/swagger-ui", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
