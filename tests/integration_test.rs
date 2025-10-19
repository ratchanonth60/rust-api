use reqwest;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_full_flow() {
    println!("Waiting for server to start...");
    sleep(Duration::from_secs(3)).await;

    let client = reqwest::Client::new();
    let unique_username = format!("testuser_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
    let unique_email = format!("{}@example.com", unique_username);

    // 1. Register
    let register_res = client.post("http://127.0.0.1:3000/users")
        .json(&json!({
            "username": unique_username,
            "email": unique_email,
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(register_res.status(), reqwest::StatusCode::CREATED);

    // 2. Register with duplicate username
    let register_again_res = client.post("http://127.0.0.1:3000/users")
        .json(&json!({
            "username": unique_username,
            "email": "another@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(register_again_res.status(), reqwest::StatusCode::CONFLICT);


    // 3. Login
    let login_res = client.post("http://127.0.0.1:3000/login")
        .json(&json!({
            "username": unique_username,
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(login_res.status(), reqwest::StatusCode::OK);
    let login_json: serde_json::Value = login_res.json().await.unwrap();
    let access_token = login_json["token"]["access_token"].as_str().unwrap();
    let refresh_token = login_json["token"]["refresh_token"].as_str().unwrap();


    // 4. Get Profile
    let profile_res = client.get("http://127.0.0.1:3000/profile")
        .bearer_auth(access_token)
        .send()
        .await
        .unwrap();
    assert_eq!(profile_res.status(), reqwest::StatusCode::OK);
    let profile_json: serde_json::Value = profile_res.json().await.unwrap();
    assert_eq!(profile_json["username"], unique_username);


    // 5. Get Profile without token
    let profile_no_token_res = client.get("http://127.0.0.1:3000/profile")
        .send()
        .await
        .unwrap();
    assert_eq!(profile_no_token_res.status(), reqwest::StatusCode::UNAUTHORIZED);


    // 6. Refresh token
    let refresh_res = client.post("http://127.0.0.1:3000/refresh")
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(refresh_res.status(), reqwest::StatusCode::OK);
    let refresh_json: serde_json::Value = refresh_res.json().await.unwrap();
    assert!(refresh_json["access_token"].as_str().is_some());

}
