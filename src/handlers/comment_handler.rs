use crate::{    errors::AppError,    models::{        comment::{Comment, CreateCommentPayload},        jwt::Claims,    },
    state::AppState,
};
use axum::{extract::{State, Path}, http::StatusCode, Json};
use validator::Validate;
use std::sync::Arc;



#[utoipa::path(
    post,
    path = "/posts/{id}/comments",
    request_body = CreateCommentPayload,
    params(
        ("id" = i32, Path, description = "Post ID")
    ),
    responses(
        (status = 201, description = "Comment created successfully", body = Comment),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body = inline(serde_json::Value))
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_comment(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(post_id_path): Path<i32>,
    Json(new_comment): Json<CreateCommentPayload>,
) -> Result<(StatusCode, Json<Comment>), AppError> {
    new_comment.validate()?;
    let created_comment = state.comment_usecase.create_comment(new_comment, claims.sub, post_id_path).await?;
    Ok((StatusCode::CREATED, Json(created_comment)))
}

#[utoipa::path(
    get,
    path = "/posts/{id}/comments",
    params(
        ("id" = i32, Path, description = "Post ID")
    ),
    responses(
        (status = 200, description = "List of comments for a post", body = Vec<Comment>),
        (status = 500, description = "Internal Server Error", body = inline(serde_json::Value))
    )
)]
pub async fn get_comments_for_post(
    State(state): State<Arc<AppState>>,
    Path(post_id_path): Path<i32>,
) -> Result<Json<Vec<Comment>>, AppError> {
    let comments_for_post = state.comment_usecase.get_comments_for_post(post_id_path).await?;
    Ok(Json(comments_for_post))
}

#[utoipa::path(
    patch,
    path = "/comments/{id}",
    request_body = CreateCommentPayload,
    params(
        ("id" = i32, Path, description = "Comment ID")
    ),
    responses(
        (status = 200, description = "Comment updated successfully", body = Comment),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Comment not found"),
        (status = 500, description = "Internal Server Error", body = inline(serde_json::Value))
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_comment(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(comment_id_path): Path<i32>,
    Json(update_payload): Json<CreateCommentPayload>,
) -> Result<Json<Comment>, AppError> {
    update_payload.validate()?;
    let updated_comment = state.comment_usecase.update_comment(comment_id_path, update_payload, claims.sub).await?;
    Ok(Json(updated_comment))
}

#[utoipa::path(
    delete,
    path = "/comments/{id}",
    params(
        ("id" = i32, Path, description = "Comment ID")
    ),
    responses(
        (status = 204, description = "Comment deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Comment not found"),
        (status = 500, description = "Internal Server Error", body = inline(serde_json::Value))
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_comment(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(comment_id_path): Path<i32>,
) -> Result<StatusCode, AppError> {
    state.comment_usecase.delete_comment(comment_id_path, claims.sub).await?;
    Ok(StatusCode::NO_CONTENT)
}
