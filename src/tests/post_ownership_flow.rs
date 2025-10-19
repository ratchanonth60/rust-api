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

async fn create_category(client: &TestServer, admin_token: &str) -> i32 {
    let response = client
        .post("/categories")
        .add_header("Authorization", format!("Bearer {}", admin_token))
        .json(&json!({ "name": "Test Category", "slug": "test-category" }))
        .send()
        .await;
    response.assert_status(StatusCode::CREATED);
    response.json().unwrap()["id"].as_i64().unwrap() as i32
}

async fn register_admin_and_login(client: &TestServer) -> (String, String) {
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
async fn test_create_post() {
    let app = common::spawn_app().await;
    let client = TestServer::new(app).unwrap();

    let (user_token, _) = register_and_login(&client, "postuser", "post@example.com", "password123").await;
    let (admin_token, _) = register_admin_and_login(&client).await;
    let category_id = create_category(&client, &admin_token).await;

    // Test create post unauthenticated (should fail)
    let response = client
        .post("/posts")
        .json(&json!({ "title": "Unauth Post", "content": "Content", "category_id": category_id }))
        .send()
        .await;
    response.assert_status(StatusCode::UNAUTHORIZED);

    // Test create post authenticated (should succeed)
    let response = client
        .post("/posts")
        .add_header("Authorization", format!("Bearer {}", user_token))
        .json(&json!({ "title": "Auth Post", "content": "Content", "category_id": category_id }))
        .send()
        .await;
    response.assert_status(StatusCode::CREATED);
    response.assert_json_contains(json!({
        "title": "Auth Post",
        "content": "Content",
        "category_id": category_id,
        "user_id": 1 // Assuming user ID is 1
    }));
}

#[tokio::test]
async fn test_update_post_ownership() {
    let app = common::spawn_app().await;
    let client = TestServer::new(app).unwrap();

    let (owner_token, _) = register_and_login(&client, "owner", "owner@example.com", "password123").await;
    let (other_user_token, _) = register_and_login(&client, "other", "other@example.com", "password123").await;
    let (admin_token, _) = register_admin_and_login(&client).await;
    let category_id = create_category(&client, &admin_token).await;

    // Create a post as owner
    let create_response = client
        .post("/posts")
        .add_header("Authorization", format!("Bearer {}", owner_token))
        .json(&json!({ "title": "Owner Post", "content": "Content", "category_id": category_id }))
        .send()
        .await;
    create_response.assert_status(StatusCode::CREATED);
    let post_id = create_response.json().unwrap()["id"].as_i64().unwrap() as i32;

    // Test update post as other user (should be forbidden)
    let response = client
        .patch(&format!("/posts/{}", post_id))
        .add_header("Authorization", format!("Bearer {}", other_user_token))
        .json(&json!({ "title": "Updated by Other" }))
        .send()
        .await;
    response.assert_status(StatusCode::FORBIDDEN);

    // Test update post as owner (should succeed)
    let response = client
        .patch(&format!("/posts/{}", post_id))
        .add_header("Authorization", format!("Bearer {}", owner_token))
        .json(&json!({ "title": "Updated by Owner" }))
        .send()
        .await;
    response.assert_status(StatusCode::OK);
    response.assert_json_contains(json!({
        "title": "Updated by Owner"
    }));
}

#[tokio::test]
async fn test_delete_post_ownership() {
    let app = common::spawn_app().await;
    let client = TestServer::new(app).unwrap();

    let (owner_token, _) = register_and_login(&client, "owner_del", "owner_del@example.com", "password123").await;
    let (other_user_token, _) = register_and_login(&client, "other_del", "other_del@example.com", "password123").await;
    let (admin_token, _) = register_admin_and_login(&client).await;
    let category_id = create_category(&client, &admin_token).await;

    // Create a post as owner
    let create_response = client
        .post("/posts")
        .add_header("Authorization", format!("Bearer {}", owner_token))
        .json(&json!({ "title": "Post to Delete", "content": "Content", "category_id": category_id }))
        .send()
        .await;
    create_response.assert_status(StatusCode::CREATED);
    let post_id = create_response.json().unwrap()["id"].as_i64().unwrap() as i32;

    // Test delete post as other user (should be forbidden)
    let response = client
        .delete(&format!("/posts/{}", post_id))
        .add_header("Authorization", format!("Bearer {}", other_user_token))
        .send()
        .await;
    response.assert_status(StatusCode::FORBIDDEN);

    // Test delete post as owner (should succeed)
    let response = client
        .delete(&format!("/posts/{}", post_id))
        .add_header("Authorization", format!("Bearer {}", owner_token))
        .send()
        .await;
    response.assert_status(StatusCode::NO_CONTENT);

    // Verify post is deleted
    let response = client.get(&format!("/posts/{}", post_id)).send().await;
    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_post_as_admin() {
    let app = common::spawn_app().await;
    let client = TestServer::new(app).unwrap();

    let (owner_token, _) = register_and_login(&client, "owner_admin", "owner_admin@example.com", "password123").await;
    let (admin_token, _) = register_admin_and_login(&client).await;
    let category_id = create_category(&client, &admin_token).await;

    // Create a post as owner
    let create_response = client
        .post("/posts")
        .add_header("Authorization", format!("Bearer {}", owner_token))
        .json(&json!({ "title": "Admin Delete Post", "content": "Content", "category_id": category_id }))
        .send()
        .await;
    create_response.assert_status(StatusCode::CREATED);
    let post_id = create_response.json().unwrap()["id"].as_i64().unwrap() as i32;

    // Test delete post as admin (should succeed even if not owner)
    let response = client
        .delete(&format!("/posts/{}", post_id))
        .add_header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await;
    response.assert_status(StatusCode::NO_CONTENT);

    // Verify post is deleted
    let response = client.get(&format!("/posts/{}", post_id)).send().await;
    response.assert_status(StatusCode::NOT_FOUND);
}
