use axum::{routing::post, Router};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// ประกาศ modules ตามโครงสร้าง directory
mod db;
mod errors;
mod handlers;
mod models;
mod schema;

#[tokio::main]
async fn main() {
    // โหลดตัวแปรจาก .env
    dotenvy::dotenv().ok();

    // ตั้งค่า logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "axum_diesel_demo=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // สร้าง Connection Pool
    let db_pool = db::connect::establish_connection();

    // Import handlers ที่เราต้องการใช้งาน
    use handlers::user_handler;

    // สร้าง Router และกำหนด Routes ทั้งหมด
    let app = Router::new()
        // Route สำหรับ Users
        .route("/users", post(user_handler::create_user))
        .with_state(db_pool);

    // รันเซิร์ฟเวอร์
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

