use axum::{http::StatusCode, Router};
use axum_test::TestServer;
use serde_json::json;

mod common;

#[tokio::test]
async fn test_register_success_and_fail() {
    let app = common::spawn_app().await;
    let client = TestServer::new(app).unwrap();

    // Test successful registration
    let response = client
        .post("/users")
        .json(&json!({ "username": "testuser", "email": "test@example.com", "password": "password123" }))
        .send()
        .await;

    response.assert_status(StatusCode::CREATED);
    response.assert_json(&json!({
        "id": 1,
        "username": "testuser",
        "email": "test@example.com",
        "created_at": response.json().unwrap()["created_at"],
        "role": "user"
    }));

    // Test duplicate registration (should fail)
    let response = client
        .post("/users")
        .json(&json!({ "username": "testuser", "email": "test@example.com", "password": "password123" }))
        .send()
        .await;

    response.assert_status(StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_login_success_and_fail() {
    let app = common::spawn_app().await;
    let client = TestServer::new(app).unwrap();

    // Register a user first
    client
        .post("/users")
        .json(&json!({ "username": "loginuser", "email": "login@example.com", "password": "password123" }))
        .send()
        .await
        .assert_status(StatusCode::CREATED);

    // Test successful login
    let response = client
        .post("/login")
        .json(&json!({ "username": "loginuser", "password": "password123" }))
        .send()
        .await;

    response.assert_status(StatusCode::OK);
    response.assert_json_contains(json!({
        "access_token": serde_json::Value::String("".to_string()), // Check for presence, not value
        "refresh_token": serde_json::Value::String("".to_string()),
    }));

    // Test failed login (wrong password)
    let response = client
        .post("/login")
        .json(&json!({ "username": "loginuser", "password": "wrongpassword" }))
        .send()
        .await;

    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_refresh_token() {
    let app = common::spawn_app().await;
    let client = TestServer::new(app).unwrap();

    // Register a user and log in to get tokens
    client
        .post("/users")
        .json(&json!({ "username": "refreshuser", "email": "refresh@example.com", "password": "password123" }))
        .send()
        .await
        .assert_status(StatusCode::CREATED);

    let login_response = client
        .post("/login")
        .json(&json!({ "username": "refreshuser", "password": "password123" }))
        .send()
        .await;

    login_response.assert_status(StatusCode::OK);
    let refresh_token = login_response.json().unwrap()["refresh_token"].as_str().unwrap().to_string();

    // Test successful token refresh
    let refresh_response = client
        .post("/refresh")
        .json(&json!({ "refresh_token": refresh_token }))
        .send()
        .await;

    refresh_response.assert_status(StatusCode::OK);
    refresh_response.assert_json_contains(json!({
        "access_token": serde_json::Value::String("".to_string()),
        "refresh_token": serde_json::Value::String("".to_string()),
    }));

    // Test failed token refresh (invalid token)
    let invalid_refresh_response = client
        .post("/refresh")
        .json(&json!({ "refresh_token": "invalidtoken" }))
        .send()
        .await;

    invalid_refresh_response.assert_status(StatusCode::UNAUTHORIZED);
}
