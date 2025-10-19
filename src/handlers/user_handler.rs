use crate::{ 
    errors::AppError,
    models::user::{ChangePasswordRequest, CreateUser, UpdateUser, User},
    state::AppState,
};
use axum::{extract::State, http::StatusCode, Json};
use validator::Validate;
use std::sync::Arc;

// Handler สำหรับ POST /users
#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUser,
    responses(
        (status = 201, description = "User created successfully", body = User),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "Conflict"),
        (status = 500, description = "Internal Server Error", body = inline(serde_json::Value))
    )
)]
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(new_user): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), AppError> {
    new_user.validate()?;
    let created_user = state.user_usecase.create_user(new_user).await?;
    Ok((StatusCode::CREATED, Json(created_user)))
}

#[utoipa::path(
    get,
    path = "/users/profile",
    responses(
        (status = 200, description = "User profile retrieved successfully", body = User),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_profile(
    State(state): State<Arc<AppState>>,
    claims: crate::models::jwt::Claims,
) -> Result<Json<User>, AppError> {
    let user = state.user_usecase.get_profile(claims.sub).await?;
    Ok(Json(user))
}


#[utoipa::path(
    patch,
    path = "/profile",
    request_body = UpdateUser,
    responses(
        (status = 200, description = "User profile updated successfully", body = User),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error or Duplicate Entry", body = inline(serde_json::Value))
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_profile(
    State(state): State<Arc<AppState>>,
    claims: crate::models::jwt::Claims,
    Json(update_user): Json<UpdateUser>,
) -> Result<Json<User>, AppError> {
    let updated_user = state.user_usecase.update_profile(claims.sub, update_user).await?;
    Ok(Json(updated_user))
}

#[utoipa::path(
    put,
    path = "/profile/password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Invalid old password"),
        (status = 500, description = "Internal Server Error", body = inline(serde_json::Value))
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn change_password(
    State(state): State<Arc<AppState>>,
    claims: crate::models::jwt::Claims,
    Json(password_data): Json<ChangePasswordRequest>,
) -> Result<StatusCode, AppError> {
    state.user_usecase.change_password(claims.sub, password_data).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    delete,
    path = "/profile",
    responses(
        (status = 204, description = "User profile deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body = inline(serde_json::Value))
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_profile(
    State(state): State<Arc<AppState>>,
    claims: crate::models::jwt::Claims,
) -> Result<StatusCode, AppError> {
    state.user_usecase.delete_profile(claims.sub).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "List of all users", body = Vec<User>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_all_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<User>>, AppError> {
    let all_users = state.user_usecase.get_all_users().await?;
    Ok(Json(all_users))
}

#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "User deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "User not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_user_by_id(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(user_id): axum::extract::Path<i32>,
) -> Result<StatusCode, AppError> {
    state.user_usecase.delete_user_by_id(user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}