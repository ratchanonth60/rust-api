use crate::{
    errors::AppError,
    models::{
        jwt::{RefreshTokenPayload},
        ForgotPasswordRequest, LoginRequest, ResetPasswordRequest,
    },
    state::AppState,
};
use axum::{extract::State, Json};
use serde_json::json;
use validator::Validate;
use std::sync::Arc;

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
    State(state): State<Arc<AppState>>,
    Json(login_user): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    login_user.validate()?;
    let token = state.auth_usecase.login(login_user).await?;
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
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let new_access_token = state.auth_usecase.refresh_access_token(payload).await?;
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
    State(state): State<Arc<AppState>>,
    Json(forgot_password_request): Json<ForgotPasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    state.auth_usecase.forgot_password(forgot_password_request).await?;
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
    State(state): State<Arc<AppState>>,
    Json(reset_password_request): Json<ResetPasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    state.auth_usecase.reset_password(reset_password_request).await?;
    Ok(Json(json!({ "message": "Password reset successful" })))
}
