use crate::{
    errors::AppError,
    models::category::{Category, CreateCategory},
    state::AppState,
};
use axum::{extract::State, http::StatusCode, Json};
use validator::Validate;
use std::sync::Arc;


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
    State(state): State<Arc<AppState>>,
    Json(new_category): Json<CreateCategory>,
) -> Result<(StatusCode, Json<Category>), AppError> {
    new_category.validate()?;
    let created_category = state.category_usecase.create_category(new_category).await?;
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
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Category>>, AppError> {
    let all_categories = state.category_usecase.get_all_categories().await?;
    Ok(Json(all_categories))
}
