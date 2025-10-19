use axum::{http::StatusCode, Router};
use axum_test::TestServer;
use serde_json::json;

mod common;

async fn register_and_login(client: &TestServer, username: &str, email: &str, password: &str) -> (String, String) {
    client
        .post("/users")
        .json(&json!({ "username": username, "email": email, "password": password }))
        .send()
        .await
        .assert_status(StatusCode::CREATED);

    let login_response = client
        .post("/login")
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await;

    login_response.assert_status(StatusCode::OK);
    let access_token = login_response.json().unwrap()["access_token"].as_str().unwrap().to_string();
    let refresh_token = login_response.json().unwrap()["refresh_token"].as_str().unwrap().to_string();
    (access_token, refresh_token)
}

async fn register_admin_and_login(client: &TestServer) -> (String, String) {
    // Create an admin user directly in the database for testing purposes
    let app = common::spawn_app().await;
    let mut conn = app.state().db_pool.get().expect("Failed to get a connection");
    let hashed_password = rust_api::security::hash_password("adminpassword".to_string()).await.unwrap();

    diesel::sql_query(format!(
        "INSERT INTO users (username, email, password, role) VALUES ('adminuser', 'admin@example.com', '{}', 'admin')",
        hashed_password
    ))
    .execute(&mut conn)
    .expect("Failed to create admin user");

    register_and_login(client, "adminuser", "admin@example.com", "adminpassword").await
}

#[tokio::test]
async fn test_admin_routes_as_user() {
    let app = common::spawn_app().await;
    let client = TestServer::new(app).unwrap();

    let (user_access_token, _) = register_and_login(&client, "normaluser", "user@example.com", "password123").await;

    // Attempt to create a category as a normal user (should be forbidden)
    let response = client
        .post("/categories")
        .add_header("Authorization", format!("Bearer {}", user_access_token))
        .json(&json!({ "name": "Test Category", "slug": "test-category" }))
        .send()
        .await;

    response.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_admin_routes_as_admin() {
    let app = common::spawn_app().await;
    let client = TestServer::new(app).unwrap();

    let (admin_access_token, _) = register_admin_and_login(&client).await;

    // Attempt to create a category as an admin (should succeed)
    let response = client
        .post("/categories")
        .add_header("Authorization", format!("Bearer {}", admin_access_token))
        .json(&json!({ "name": "Admin Category", "slug": "admin-category" }))
        .send()
        .await;

    response.assert_status(StatusCode::CREATED);
    response.assert_json_contains(json!({
        "name": "Admin Category",
        "slug": "admin-category"
    }));
}
