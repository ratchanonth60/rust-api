use std::sync::Once;

use axum::{body::Body, http::Request, Router};
use diesel::{Connection, PgConnection, RunQueryDsl};
use once_cell::sync::Lazy;
use rust_api::{config::AppConfig, db::connect, routes, state::AppState};
use tower::ServiceExt; // for `app.oneshot()`
use diesel_migrations::{embed_migrations, MigrationHarness};

embed_migrations!();

static TRACING: Once = Once::new();

pub fn setup_tracing() {
    TRACING.call_once(|| {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    });
}

static TEST_DATABASE_URL: Lazy<String> = Lazy::new(|| {
    dotenvy::dotenv().ok();
    let mut url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    url.push_str("_test"); // Use a separate test database
    url
});

pub async fn spawn_app() -> Router {
    setup_tracing();

    let config = AppConfig {
        database_url: TEST_DATABASE_URL.to_string(),
        jwt_secret: "super-secret-jwt-key-for-testing-only".to_string(), // Use a test secret
        server_host: "127.0.0.1".to_string(),
        server_port: 0, // Let the OS assign a free port
    };

    // Run migrations
    let mut conn = PgConnection::establish(&config.database_url)
        .expect("Failed to connect to test database");
    conn.run_pending_migrations(&diesel_migrations::MIGRATIONS).expect("Failed to run migrations");

    // Truncate tables (clean up between tests)
    // NOTE: This assumes a specific order of table creation/deletion due to foreign keys.
    // For a more robust solution, consider disabling foreign key checks or dropping/recreating the schema.
    conn.transaction::<(), diesel::result::Error, _>(|conn| {
        diesel::sql_query("TRUNCATE TABLE comments RESTART IDENTITY CASCADE").execute(conn)?;
        diesel::sql_query("TRUNCATE TABLE posts RESTART IDENTITY CASCADE").execute(conn)?;
        diesel::sql_query("TRUNCATE TABLE categories RESTART IDENTITY CASCADE").execute(conn)?;
        diesel::sql_query("TRUNCATE TABLE users RESTART IDENTITY CASCADE").execute(conn)?;
        Ok(())
    })
    .expect("Failed to truncate tables");


    let db_pool = connect::establish_connection(&config.database_url);

    let app_state = AppState {
        db_pool,
        config: config.clone(),
    };

    routes::create_router(app_state)
}
