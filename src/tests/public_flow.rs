use axum::{http::StatusCode, Router};
use axum_test::TestServer;
use serde_json::json;

mod common;

#[tokio::test]
async fn test_get_public_routes() {
    let app = common::spawn_app().await;
    let client = TestServer::new(app).unwrap();

    // Test GET /categories (should succeed without authentication)
    let response = client.get("/categories").send().await;
    response.assert_status(StatusCode::OK);
    response.assert_json(&json!([])); // Assuming no categories initially

    // Test GET /posts (should succeed without authentication)
    let response = client.get("/posts").send().await;
    response.assert_status(StatusCode::OK);
    response.assert_json(&json!({
        "items": [],
        "total_pages": 0,
        "page": 1,
        "per_page": 10
    })); // Assuming no posts initially
}
