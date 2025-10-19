use crate::{    errors::AppError,    models::{        comment::{Comment, CreateCommentPayload},        jwt::Claims,    },    state::AppState,};
use axum::{extract::{State, Path}, http::StatusCode, Json};
use diesel::prelude::*;
use validator::Validate;



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
    State(state): State<AppState>,
    claims: Claims,
    Path(post_id_path): Path<i32>,
    Json(new_comment): Json<CreateCommentPayload>,
) -> Result<(StatusCode, Json<Comment>), AppError> {
    new_comment.validate()?;

    let mut conn = state.db_pool.get().expect("Failed to get a connection");

    let created_comment = tokio::task::spawn_blocking(move || {
        use crate::schema::comments::dsl::*;
        let comment_data = (
            content.eq(&new_comment.content),
            user_id.eq(claims.sub),
            post_id.eq(post_id_path),
        );
        diesel::insert_into(comments)
            .values(comment_data)
            .returning(Comment::as_returning())
            .get_result(&mut conn)
    })
    .await
    .unwrap()?;

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
    State(state): State<AppState>,
    Path(post_id_path): Path<i32>,
) -> Result<Json<Vec<Comment>>, AppError> {
    let mut conn = state.db_pool.get().expect("Failed to get a connection");
    let comments_for_post = tokio::task::spawn_blocking(move || {
        use crate::schema::comments::dsl::*;
        comments.filter(post_id.eq(post_id_path)).select(Comment::as_select()).load(&mut conn)
    })
    .await
    .unwrap()?;
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
    State(state): State<AppState>,
    claims: Claims,
    Path(comment_id_path): Path<i32>,
    Json(update_payload): Json<CreateCommentPayload>,
) -> Result<Json<Comment>, AppError> {
    update_payload.validate()?;

    let mut conn = state.db_pool.get().expect("Failed to get a connection");

    let updated_comment = tokio::task::spawn_blocking(move || {
        use crate::schema::comments::dsl::*;

        let comment_to_update = comments.find(comment_id_path).select(Comment::as_select()).first(&mut conn)?;

        if comment_to_update.user_id != claims.sub {
            return Err(AppError::Forbidden);
        }

        Ok(diesel::update(comments.find(comment_id_path))
            .set(content.eq(&update_payload.content))
            .returning(Comment::as_returning())
            .get_result(&mut conn)?)
    })
    .await
    .unwrap()?;

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
    State(state): State<AppState>,
    claims: Claims,
    Path(comment_id_path): Path<i32>,
) -> Result<StatusCode, AppError> {
    let mut conn = state.db_pool.get().expect("Failed to get a connection");

    let num_deleted = tokio::task::spawn_blocking(move || {
        use crate::schema::comments::dsl::*;
        use crate::schema::users::dsl as user_dsl;

        let comment_to_delete = comments.find(comment_id_path).select(Comment::as_select()).first(&mut conn)?;
        let user = user_dsl::users.find(claims.sub).select(crate::models::user::User::as_select()).first(&mut conn)?;

        if comment_to_delete.user_id != claims.sub && user.role != "admin" {
            return Err(AppError::Forbidden);
        }

        Ok(diesel::delete(comments.find(comment_id_path)).execute(&mut conn)?)
    })
    .await
    .unwrap()?;

    if num_deleted == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
