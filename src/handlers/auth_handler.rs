use crate::{
    errors::AppError,
    models::{
        jwt::{RefreshTokenPayload, TokenResponse},
        LoginRequest, User,
    }, // Import models ที่เพิ่มเข้ามา
    security::{create_access_token, create_refresh_token, decode_token, verify_password}, // Import functions ที่เพิ่มเข้ามา
    state::AppState,
};
use axum::{extract::State, Json};
use diesel::prelude::*;
use serde_json::json;

#[utoipa::path(
    post,
    path = "/users/login",
    request_body = LoginUser,
    responses(
        (status = 200, description = "Login successful", body = inline(serde_json::Value)),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(login_user): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut conn = state
        .db_pool
        .get()
        .expect("failed to get db connection from pool");

    // ดึง user พร้อม password hash จาก database
    let user_with_password = tokio::task::spawn_blocking(move || {
        use crate::schema::users::dsl::*;
        users
            .filter(username.eq(login_user.username))
            .select(User::as_select())
            .first(&mut conn)
    })
    .await
    .unwrap()?; // unwrap a Result from spawn_blocking

    // ตรวจสอบรหัสผ่าน
    if !verify_password(&user_with_password.password, &login_user.password).unwrap_or(false) {
        return Err(AppError::Unauthorized);
    }

    // สร้าง User object ที่ไม่มี password เพื่อส่งไปสร้าง token
    let user = User {
        id: user_with_password.id,
        username: user_with_password.username,
        email: user_with_password.email,
        password: "".to_string(), // ไม่ส่งรหัสผ่านกลับไป
        created_at: user_with_password.created_at,
    };

    // สร้าง JWT
    let access_token = create_access_token(&user, &state.config.jwt_secret)
        .map_err(|_| AppError::InternalServerError("Failed to create JWT".to_string()))?;
    let refresh_token = create_refresh_token(&user, &state.config.jwt_refresh_secret)
        .map_err(|_| AppError::InternalServerError("Failed to create JWT".to_string()))?;
    let token = TokenResponse {
        access_token,
        refresh_token: refresh_token.clone(),
    };
    Ok(Json(json!({ "token": token })))
}

#[utoipa::path(
    post,
    path = "/users/refresh",
    request_body = RefreshTokenPayload,
    responses(
        (status = 200, description = "Access token refreshed successfully", body = inline(serde_json::Value)),
        (status = 401, description = "Unauthorized or invalid refresh token")
    )
)]
pub async fn refresh_access_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    // ถอดรหัส refresh token
    let claims = decode_token(&payload.refresh_token, &state.config.jwt_refresh_secret)
        .map_err(|_| AppError::Unauthorized)?;

    let mut conn = state
        .db_pool
        .get()
        .expect("failed to get db connection from pool");

    // ค้นหา user จาก ID ใน token
    let user = tokio::task::spawn_blocking(move || {
        use crate::schema::users::dsl::*;
        users
            .find(claims.sub)
            .select(User::as_select())
            .first(&mut conn)
    })
    .await
    .unwrap()?;

    // สร้าง access token ใหม่
    let new_access_token = create_access_token(&user, &state.config.jwt_secret).map_err(|_| {
        AppError::InternalServerError("Failed to create new access token".to_string())
    })?;

    Ok(Json(json!({ "access_token": new_access_token })))
}
