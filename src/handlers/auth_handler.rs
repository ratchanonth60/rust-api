use crate::{
    errors::AppError,
    models::{
        jwt::{RefreshTokenPayload, TokenResponse},
        password_reset::{NewPasswordResetToken, PasswordResetToken},
        ForgotPasswordRequest, LoginRequest, ResetPasswordRequest, User,
    }, // Import models ที่เพิ่มเข้ามา
    security::{
        create_access_token, create_refresh_token, decode_token, hash_password, verify_password,
    }, // Import functions ที่เพิ่มเข้ามา
    state::AppState,
};
use axum::{extract::State, Json};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use rand::{distributions::Alphanumeric, Rng};
use serde_json::json;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/users/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = inline(serde_json::Value)),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(login_user): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    login_user.validate()?;
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
        role: user_with_password.role,
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

#[utoipa::path(
    post,
    path = "/forgot-password",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Password reset email sent"),
        (status = 404, description = "User not found")
    )
)]
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(forgot_password_request): Json<ForgotPasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_email = forgot_password_request.email;
    let db_pool = state.db_pool.clone();
    let user_email_clone_for_find = user_email.clone();

    // Find user by email
    let _user: User = tokio::task::spawn_blocking(move || {
        let mut conn = db_pool.get().expect("failed to get db connection from pool");
        use crate::schema::users::dsl::*;
        users.filter(email.eq(&user_email_clone_for_find)).first(&mut conn)
    })
    .await
    .unwrap()?;

    // Generate a random token
    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    // Store the token in the database
    let db_pool = state.db_pool.clone();
    let token_clone = token.clone();
    let user_email_clone_for_insert = user_email.clone();
    tokio::task::spawn_blocking(move || {
        let mut conn = db_pool.get().expect("failed to get db connection from pool");
        use crate::schema::password_reset_tokens::dsl::*;
        diesel::insert_into(password_reset_tokens)
            .values(&NewPasswordResetToken {
                email: &user_email_clone_for_insert,
                token: &token_clone,
            })
            .on_conflict(email)
            .do_update()
            .set(token.eq(&token_clone))
            .execute(&mut conn)
    })
    .await
    .unwrap()?;

    // In a real application, you would send an email here.
    // For this example, we'll just log it.
    println!("Password reset token for {}: {}", user_email, token);

    Ok(Json(json!({ "message": "Password reset email sent" })))
}

#[utoipa::path(
    post,
    path = "/reset-password",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successful"),
        (status = 400, description = "Invalid or expired token")
    )
)]
pub async fn reset_password(
    State(state): State<AppState>,
    Json(reset_password_request): Json<ResetPasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let request_token = reset_password_request.token;
    let db_pool = state.db_pool.clone();

    // Find the token in the database
    let reset_token: PasswordResetToken = tokio::task::spawn_blocking(move || {
        let mut conn = db_pool.get().expect("failed to get db connection from pool");
        use crate::schema::password_reset_tokens::dsl::*;
        password_reset_tokens
            .filter(token.eq(&request_token))
            .first(&mut conn)
    })
    .await
    .unwrap()?;

    // Check if the token is expired (e.g., 1 hour)
    if Utc::now().naive_utc() - reset_token.created_at > Duration::hours(1) {
        let db_pool = state.db_pool.clone();
        let token_to_delete = reset_token.token.clone();
        // Delete the expired token
        tokio::task::spawn_blocking(move || {
            let mut conn = db_pool.get().expect("failed to get db connection from pool");
            use crate::schema::password_reset_tokens::dsl::*;
            diesel::delete(password_reset_tokens.filter(token.eq(&token_to_delete)))
                .execute(&mut conn)
        })
        .await
        .unwrap()?;
        return Err(AppError::BadRequest("Invalid or expired token".to_string()));
    }

    // Hash the new password
    let new_password_hash = hash_password(reset_password_request.new_password)
        .await
        .map_err(|e| AppError::InternalServerError(e))?;

    // Update the user's password
    let db_pool = state.db_pool.clone();
    let user_email = reset_token.email.clone();
    tokio::task::spawn_blocking(move || {
        let mut conn = db_pool.get().expect("failed to get db connection from pool");
        use crate::schema::users::dsl::*;
        diesel::update(users.filter(email.eq(user_email)))
            .set(password.eq(new_password_hash))
            .execute(&mut conn)
    })
    .await
    .unwrap()?;

    // Delete the reset token
    let db_pool = state.db_pool.clone();
    tokio::task::spawn_blocking(move || {
        let mut conn = db_pool.get().expect("failed to get db connection from pool");
        use crate::schema::password_reset_tokens::dsl::*;
        diesel::delete(password_reset_tokens.filter(token.eq(&reset_token.token)))
            .execute(&mut conn)
    })
    .await
    .unwrap()?;

    Ok(Json(json!({ "message": "Password reset successful" })))
}
