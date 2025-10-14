use crate::{
    errors::AppError,
    models::{CreateUser, User}, // Import model ที่เราสร้าง
    security::hash_password,
    state::AppState,
};
use axum::{extract::State, http::StatusCode, Json};
use diesel::prelude::*;

// Handler สำหรับ POST /users
#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUser,
    responses(
        (status = 201, description = "User created successfully", body = User),
        (status = 500, description = "Internal Server Error or Duplicate Entry", body = inline(serde_json::Value))
    )
)]
pub async fn create_user(
    State(state): State<AppState>,
    Json(mut new_user): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), AppError> {
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
    claims: crate::middlewars::auth::Claims,
) -> Result<Json<User>, AppError> {
    let mut conn = state.db_pool.get().expect("Failed to get a connection");
    let user_id = claims.sub;
    let user = tokio::task::spawn_blocking(move || {
        use crate::schema::users::dsl::*;
        users.filter(id.eq(user_id)).first::<User>(&mut conn)
    })
    .await
    .unwrap()?;
    Ok(Json(user))
}
