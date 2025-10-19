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
        .json(&json!({ "name": "Comment Category", "slug": "comment-category" }))
        .send()
        .await;
    response.assert_status(StatusCode::CREATED);
    response.json().unwrap()["id"].as_i64().unwrap() as i32
}

async fn create_post(client: &TestServer, user_token: &str, category_id: i32) -> i32 {
    let response = client
        .post("/posts")
        .add_header("Authorization", format!("Bearer {}", user_token))
        .json(&json!({ "title": "Comment Post", "content": "Content for comments", "category_id": category_id }))
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
        "INSERT INTO users (username, email, password, role) VALUES ('admincomment', 'admincomment@example.com', '{}', 'admin')",
        hashed_password
    ))
    .execute(&mut conn)
    .expect("Failed to create admin user");

    register_and_login(client, "admincomment", "admincomment@example.com", "adminpassword").await
}

#[tokio::test]
async fn test_create_and_get_comments() {
    let app = common::spawn_app().await;
    let client = TestServer::new(app).unwrap();

    let (user_token, _) = register_and_login(&client, "commenter", "commenter@example.com", "password123").await;
    let (admin_token, _) = register_admin_and_login(&client).await;
    let category_id = create_category(&client, &admin_token).await;
    let post_id = create_post(&client, &user_token, category_id).await;

    // Test create comment unauthenticated (should fail)
    let response = client
        .post(&format!("/posts/{}/comments", post_id))
        .json(&json!({ "content": "Unauth comment" }))
        .send()
        .await;
    response.assert_status(StatusCode::UNAUTHORIZED);

    // Test create comment authenticated (should succeed)
    let response = client
        .post(&format!("/posts/{}/comments", post_id))
        .add_header("Authorization", format!("Bearer {}", user_token))
        .json(&json!({ "content": "First comment" }))
        .send()
        .await;
    response.assert_status(StatusCode::CREATED);
    response.assert_json_contains(json!({
        "content": "First comment",
        "user_id": 1,
        "post_id": post_id
    }));

    // Test get comments for post (should succeed)
    let response = client
        .get(&format!("/posts/{}/comments", post_id))
        .send()
        .await;
    response.assert_status(StatusCode::OK);
    response.assert_json_contains(json!([
        {
            "content": "First comment",
            "user_id": 1,
            "post_id": post_id
        }
    ]));
}

#[tokio::test]
async fn test_delete_comment_ownership() {
    let app = common::spawn_app().await;
    let client = TestServer::new(app).unwrap();

    let (owner_token, _) = register_and_login(&client, "commentowner", "commentowner@example.com", "password123").await;
    let (other_user_token, _) = register_and_login(&client, "commentother", "commentother@example.com", "password123").await;
    let (admin_token, _) = register_admin_and_login(&client).await;
    let category_id = create_category(&client, &admin_token).await;
    let post_id = create_post(&client, &owner_token, category_id).await;

    // Create a comment as owner
    let create_response = client
        .post(&format!("/posts/{}/comments", post_id))
        .add_header("Authorization", format!("Bearer {}", owner_token))
        .json(&json!({ "content": "Comment to delete" }))
        .send()
        .await;
    create_response.assert_status(StatusCode::CREATED);
    let comment_id = create_response.json().unwrap()["id"].as_i64().unwrap() as i32;

    // Test delete comment as other user (should be forbidden)
    let response = client
        .delete(&format!("/comments/{}", comment_id))
        .add_header("Authorization", format!("Bearer {}", other_user_token))
        .send()
        .await;
    response.assert_status(StatusCode::FORBIDDEN);

    // Test delete comment as owner (should succeed)
    let response = client
        .delete(&format!("/comments/{}", comment_id))
        .add_header("Authorization", format!("Bearer {}", owner_token))
        .send()
        .await;
    response.assert_status(StatusCode::NO_CONTENT);

    // Verify comment is deleted
    let response = client.get(&format!("/posts/{}/comments", post_id)).send().await;
    response.assert_status(StatusCode::OK);
    response.assert_json(&json!([]));

    // Recreate comment for admin test
    let create_response = client
        .post(&format!("/posts/{}/comments", post_id))
        .add_header("Authorization", format!("Bearer {}", owner_token))
        .json(&json!({ "content": "Comment for admin delete" }))
        .send()
        .await;
    create_response.assert_status(StatusCode::CREATED);
    let comment_id_for_admin = create_response.json().unwrap()["id"].as_i64().unwrap() as i32;

    // Test delete comment as admin (should succeed even if not owner)
    let response = client
        .delete(&format!("/comments/{}", comment_id_for_admin))
        .add_header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await;
    response.assert_status(StatusCode::NO_CONTENT);

    // Verify comment is deleted
    let response = client.get(&format!("/posts/{}/comments", post_id)).send().await;
    response.assert_status(StatusCode::OK);
    response.assert_json(&json!([]));
}
