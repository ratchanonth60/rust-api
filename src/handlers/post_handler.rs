use crate::{    errors::AppError,    models::{        jwt::Claims,        pagination::Paginated,        post::{CreatePostPayload, Post, UpdatePostPayload},    },
    state::AppState,
};
use axum::{extract::{Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use validator::Validate;
use std::sync::Arc;



#[derive(Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    10
}

#[utoipa::path(
    post,
    path = "/posts",
    request_body = CreatePostPayload,
    responses(
        (status = 201, description = "Post created successfully", body = Post),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body = inline(serde_json::Value))
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_post(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Json(new_post): Json<CreatePostPayload>,
) -> Result<(StatusCode, Json<Post>), AppError> {
    new_post.validate()?;
    let created_post = state.post_usecase.create_post(new_post, claims.sub).await?;
    Ok((StatusCode::CREATED, Json(created_post)))
}

#[utoipa::path(
    get,
    path = "/posts",
    params(
        ("page" = Option<i64>, Query, description = "Page number"),
        ("per_page" = Option<i64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "List of all posts", body = Paginated<Post>),
        (status = 500, description = "Internal Server Error", body = inline(serde_json::Value))
    )
)]
pub async fn get_posts(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Paginated<Post>>, AppError> {
    let paginated_posts = state.post_usecase.get_posts(params.page, params.per_page).await?;
    Ok(Json(paginated_posts))
}

#[utoipa::path(
    get,
    path = "/posts/{id}",
    params(
        ("id" = i32, Path, description = "Post ID")
    ),
    responses(
        (status = 200, description = "Post retrieved successfully", body = Post),
        (status = 404, description = "Post not found"),
        (status = 500, description = "Internal Server Error", body = inline(serde_json::Value))
    )
)]
pub async fn get_post_by_id(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(post_id): axum::extract::Path<i32>,
) -> Result<Json<Post>, AppError> {
    let post = state.post_usecase.get_post_by_id(post_id).await?;
    Ok(Json(post))
}

#[utoipa::path(
    get,
    path = "/categories/{slug}/posts",
    params(
        ("slug" = String, Path, description = "Category Slug")
    ),
    responses(
        (status = 200, description = "List of posts in a category", body = Vec<Post>),
        (status = 404, description = "Category not found"),
        (status = 500, description = "Internal Server Error", body = inline(serde_json::Value))
    )
)]
pub async fn get_posts_by_category(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(slug_path): axum::extract::Path<String>,
) -> Result<Json<Vec<Post>>, AppError> {
    let posts_in_category = state.post_usecase.get_posts_by_category(slug_path).await?;
    Ok(Json(posts_in_category))
}

#[utoipa::path(
    patch,
    path = "/posts/{id}",
    request_body = UpdatePostPayload,
    params(
        ("id" = i32, Path, description = "Post ID")
    ),
    responses(
        (status = 200, description = "Post updated successfully", body = Post),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Post not found"),
        (status = 500, description = "Internal Server Error", body = inline(serde_json::Value))
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_post(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    axum::extract::Path(post_id): axum::extract::Path<i32>,
    Json(update_payload): Json<UpdatePostPayload>,
) -> Result<Json<Post>, AppError> {
    update_payload.validate()?;
    let updated_post = state.post_usecase.update_post(post_id, update_payload, claims.sub).await?;
    Ok(Json(updated_post))
}

#[utoipa::path(
    delete,
    path = "/posts/{id}",
    params(
        ("id" = i32, Path, description = "Post ID")
    ),
    responses(
        (status = 204, description = "Post deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Post not found"),
        (status = 500, description = "Internal Server Error", body = inline(serde_json::Value))
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_post(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    axum::extract::Path(post_id): axum::extract::Path<i32>,
) -> Result<StatusCode, AppError> {
    state.post_usecase.delete_post(post_id, claims.sub).await?;
    Ok(StatusCode::NO_CONTENT)
}
