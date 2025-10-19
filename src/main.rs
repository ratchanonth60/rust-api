use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::Arc;

use crate::middlewars::rate_limit::RateLimiter;
use crate::repositories::{post_repository::PostRepository, user_repository::UserRepository, password_reset_token_repository::PasswordResetTokenRepository, category_repository::CategoryRepository, comment_repository::CommentRepository};
use crate::usecases::{auth_usecase::AuthUsecase, user_usecase::UserUsecase, post_usecase::PostUsecase, category_usecase::CategoryUsecase, comment_usecase::CommentUsecase};

// Declare modules
mod config;
mod db;
mod errors;
mod handlers;
mod repositories;
mod usecases;
mod middlewars;
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
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    // Create database connection pool
    let db_pool = db::connect::establish_connection(&config.database_url);

    // Run database migrations

    // Create Repositories
    let user_repo = Arc::new(UserRepository::new(db_pool.clone()));
    let post_repo = Arc::new(PostRepository::new(db_pool.clone()));
    let category_repo = Arc::new(CategoryRepository::new(db_pool.clone()));
    let comment_repo = Arc::new(CommentRepository::new(db_pool.clone()));
    let password_reset_token_repo = Arc::new(PasswordResetTokenRepository::new(db_pool.clone()));

    // Create Usecases
    let auth_usecase = Arc::new(AuthUsecase::new(user_repo.clone(), password_reset_token_repo.clone(), Arc::new(config.clone())));
    let user_usecase = Arc::new(UserUsecase::new(user_repo.clone()));
    let post_usecase = Arc::new(PostUsecase::new(post_repo.clone(), user_repo.clone()));
    let category_usecase = Arc::new(CategoryUsecase::new(category_repo.clone()));
    let comment_usecase = Arc::new(CommentUsecase::new(comment_repo.clone(), user_repo.clone()));

    // Create application state
    let app_state = state::AppState {
        config: config.clone(),
        rate_limiter: RateLimiter::new(),
        auth_usecase,
        user_usecase,
        post_usecase,
        category_usecase,
        comment_usecase,
    };

    // Create the router
    let app = routes::create_router(Arc::new(app_state));

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