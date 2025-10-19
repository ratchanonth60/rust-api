use crate::{
    errors::AppError,
    models::category::{Category, CreateCategory},
    state::AppState,
};
use axum::{extract::State, http::StatusCode, Json};
use diesel::prelude::*;
use validator::Validate;


#[utoipa::path(
    post,
    path = "/categories",
    request_body = CreateCategory,
    responses(
        (status = 201, description = "Category created successfully", body = Category),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Conflict"),
        (status = 500, description = "Internal Server Error", body = inline(serde_json::Value))
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_category(
    State(state): State<AppState>,
    Json(new_category): Json<CreateCategory>,
) -> Result<(StatusCode, Json<Category>), AppError> {
    new_category.validate()?;

    let mut conn = state.db_pool.get().expect("Failed to get a connection");

    let created_category = tokio::task::spawn_blocking(move || {
        use crate::schema::categories::dsl::*;
        diesel::insert_into(categories)
            .values(&new_category)
            .returning(Category::as_returning())
            .get_result(&mut conn)
    })
    .await
    .unwrap()?;

    Ok((StatusCode::CREATED, Json(created_category)))
}

#[utoipa::path(
    get,
    path = "/categories",
    responses(
        (status = 200, description = "List of all categories", body = Vec<Category>),
        (status = 500, description = "Internal Server Error", body = inline(serde_json::Value))
    )
)]
pub async fn get_categories(
    State(state): State<AppState>,
) -> Result<Json<Vec<Category>>, AppError> {
    let mut conn = state.db_pool.get().expect("Failed to get a connection");
    let all_categories = tokio::task::spawn_blocking(move || {
        use crate::schema::categories::dsl::*;
        categories.select(Category::as_select()).load(&mut conn)
    })
    .await
    .unwrap()?;
    Ok(Json(all_categories))
}
