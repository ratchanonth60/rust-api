use crate::{ 
    errors::AppError,
    models::user::{ChangePasswordRequest, CreateUser, UpdateUser, User}, // Import model ที่เราสร้าง
    security::{hash_password, verify_password},
    state::AppState,
};
use axum::{extract::State, http::StatusCode, Json};
use diesel::prelude::*;
use validator::Validate;

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
    State(state): State<AppState>,
    Json(mut new_user): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), AppError> {
    new_user.validate()?;
    new_user.password = hash_password(new_user.password)
        .await
        .map_err(|e| AppError::InternalServerError(e))?;

    let mut conn = state.db_pool.get().expect("Failed to get a connection");

    let created_user = tokio::task::spawn_blocking(move || {
        use crate::schema::users::dsl::*;
        diesel::insert_into(users)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(&mut conn)
    })
    .await
    .unwrap()?;

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
    State(state): State<AppState>,
    claims: crate::models::jwt::Claims,
) -> Result<Json<User>, AppError> {
    let mut conn = state.db_pool.get().expect("Failed to get a connection");
    let user_id = claims.sub;
    let user = tokio::task::spawn_blocking(move || {
        use crate::schema::users::dsl::*;
        users.filter(id.eq(user_id)).select(User::as_select()).first(&mut conn)
    })
    .await
    .unwrap()?;
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
    State(state): State<AppState>,
    claims: crate::models::jwt::Claims,
    Json(update_user): Json<UpdateUser>,
) -> Result<Json<User>, AppError> {
    let mut conn = state.db_pool.get().expect("Failed to get a connection");
    let user_id = claims.sub;

    let updated_user = tokio::task::spawn_blocking(move || {
        use crate::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(&update_user)
            .returning(User::as_returning())
            .get_result(&mut conn)
    })
    .await
    .unwrap()?;

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
    State(state): State<AppState>,
    claims: crate::models::jwt::Claims,
    Json(password_data): Json<ChangePasswordRequest>,
) -> Result<StatusCode, AppError> {
    let mut conn = state.db_pool.get().expect("Failed to get a connection");
    let user_id = claims.sub;

    let user_password: String = {
        use crate::schema::users::dsl::*;
        users
            .filter(id.eq(user_id))
            .select(password)
            .first(&mut conn)?
    };

    let is_valid = verify_password(&user_password, &password_data.old_password)
        .map_err(|_| AppError::InternalServerError("Failed to verify password".to_string()))?;

    if !is_valid {
        return Err(AppError::Unauthorized);
    }

    let new_password_hash = hash_password(password_data.new_password)
        .await
        .map_err(|_| AppError::InternalServerError("Failed to hash password".to_string()))?;

    {
        use crate::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(password.eq(new_password_hash))
            .execute(&mut conn)?;
    }

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
    State(state): State<AppState>,
    claims: crate::models::jwt::Claims,
) -> Result<StatusCode, AppError> {
    let user_id = claims.sub;
    let db_pool = state.db_pool.clone();

    let _num_deleted = tokio::task::spawn_blocking(move || {
        let mut conn = db_pool.get().expect("Failed to get a connection");
        use crate::schema::users::dsl::*;
        diesel::delete(users.filter(id.eq(user_id))).execute(&mut conn)
    })
    .await
    .unwrap()?;

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
    State(state): State<AppState>,
) -> Result<Json<Vec<User>>, AppError> {
    let mut conn = state.db_pool.get().expect("Failed to get a connection");
    let all_users = tokio::task::spawn_blocking(move || {
        use crate::schema::users::dsl::*;
        users.select(User::as_select()).load(&mut conn)
    })
    .await
    .unwrap()?;
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
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<i32>,
) -> Result<StatusCode, AppError> {
    let db_pool = state.db_pool.clone();
    let num_deleted = tokio::task::spawn_blocking(move || {
        let mut conn = db_pool.get().expect("Failed to get a connection");
        use crate::schema::users::dsl::*;
        diesel::delete(users.filter(id.eq(user_id))).execute(&mut conn)
    })
    .await
    .unwrap()?;

    if num_deleted == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}