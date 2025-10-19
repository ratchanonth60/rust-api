use axum::{http::StatusCode, Router};
use axum_test::TestServer;
use serde_json::json;

mod common;

async fn register_and_login(client: &TestServer) -> (String, String) {
    client
        .post("/users")
        .json(&json!({ "username": "profileuser", "email": "profile@example.com", "password": "password123" }))
        .send()
        .await
        .assert_status(StatusCode::CREATED);

    let login_response = client
        .post("/login")
        .json(&json!({ "username": "profileuser", "password": "password123" }))
        .send()
        .await;

    login_response.assert_status(StatusCode::OK);
    let access_token = login_response.json().unwrap()["access_token"].as_str().unwrap().to_string();
    let refresh_token = login_response.json().unwrap()["refresh_token"].as_str().unwrap().to_string();
    (access_token, refresh_token)
}

#[tokio::test]
async fn test_get_profile_unauthenticated() {
    let app = common::spawn_app().await;
    let client = TestServer::new(app).unwrap();

    let response = client.get("/profile").send().await;
    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_profile_authenticated() {
    let app = common::spawn_app().await;
    let client = TestServer::new(app).unwrap();

    let (access_token, _) = register_and_login(&client).await;

    let response = client
        .get("/profile")
        .add_header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await;

    response.assert_status(StatusCode::OK);
    response.assert_json(&json!({
        "id": 1,
        "username": "profileuser",
        "email": "profile@example.com",
        "created_at": response.json().unwrap()["created_at"],
        "role": "user"
    }));
}

#[tokio::test]
async fn test_update_profile() {
    let app = common::spawn_app().await;
    let client = TestServer::new(app).unwrap();

    let (access_token, _) = register_and_login(&client).await;

    let response = client
        .patch("/profile")
        .add_header("Authorization", format!("Bearer {}", access_token))
        .json(&json!({ "username": "updateduser", "email": "updated@example.com" }))
        .send()
        .await;

    response.assert_status(StatusCode::OK);
    response.assert_json_contains(json!({
        "username": "updateduser",
        "email": "updated@example.com"
    }));

    // Verify the update by getting the profile again
    let response = client
        .get("/profile")
        .add_header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await;
    response.assert_status(StatusCode::OK);
    response.assert_json_contains(json!({
        "username": "updateduser",
        "email": "updated@example.com"
    }));
}
