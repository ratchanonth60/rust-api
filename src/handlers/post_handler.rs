use crate::{    errors::AppError,    models::{        category::Category,        jwt::Claims,        pagination::Paginated,        post::{CreatePostPayload, Post, UpdatePostPayload},    },    state::AppState,};
use axum::{extract::{Query, State}, http::StatusCode, Json};
use diesel::prelude::*;
use serde::Deserialize;
use validator::Validate;
use diesel::BelongingToDsl;



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
    State(state): State<AppState>,
    claims: Claims,
    Json(new_post): Json<CreatePostPayload>,
) -> Result<(StatusCode, Json<Post>), AppError> {
    new_post.validate()?;

    let mut conn = state.db_pool.get().expect("Failed to get a connection");

    let created_post = tokio::task::spawn_blocking(move || {
        use crate::schema::posts::dsl::*;
        let post_data = (
            title.eq(&new_post.title),
            content.eq(&new_post.content),
            category_id.eq(new_post.category_id),
            user_id.eq(claims.sub),
        );
        diesel::insert_into(posts)
            .values(post_data)
            .returning(Post::as_returning())
            .get_result(&mut conn)
    })
    .await
    .unwrap()?;

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
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Paginated<Post>>, AppError> {
    let mut conn = state.db_pool.get().expect("Failed to get a connection");
    let (posts, total) = tokio::task::spawn_blocking(move || {
        use crate::schema::posts::dsl::*;
        let total = posts.into_boxed().count().get_result::<i64>(&mut conn)?;

        let limit = params.per_page;
        let offset = (params.page - 1) * limit;
        let result = posts.into_boxed().limit(limit).offset(offset).select(Post::as_select()).load(&mut conn)?;
        Ok::<(Vec<Post>, i64), diesel::result::Error>((result, total))
    })
    .await
    .unwrap()?;

    let total_pages = (total as f64 / params.per_page as f64).ceil() as i64;

    Ok(Json(Paginated {
        items: posts,
        total_pages,
        page: params.page,
        per_page: params.per_page,
    }))
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
    State(state): State<AppState>,
    axum::extract::Path(post_id): axum::extract::Path<i32>,
) -> Result<Json<Post>, AppError> {
    let mut conn = state.db_pool.get().expect("Failed to get a connection");
    let post = tokio::task::spawn_blocking(move || {
        use crate::schema::posts::dsl::*;
        posts.find(post_id).select(Post::as_select()).first(&mut conn)
    })
    .await
    .unwrap()?;
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
    State(state): State<AppState>,
    axum::extract::Path(slug_path): axum::extract::Path<String>,
) -> Result<Json<Vec<Post>>, AppError> {
    let mut conn = state.db_pool.get().expect("Failed to get a connection");
    let posts_in_category = tokio::task::spawn_blocking(move || {
        use crate::schema::categories::dsl as categories_dsl;

        let category = categories_dsl::categories
            .filter(categories_dsl::slug.eq(slug_path))
            .select(Category::as_select())
            .first(&mut conn)?;

        Post::belonging_to(&category)
            .select(Post::as_select())
            .load(&mut conn)
    })
    .await
    .unwrap()?;
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
    State(state): State<AppState>,
    claims: Claims,
    axum::extract::Path(post_id): axum::extract::Path<i32>,
    Json(update_payload): Json<UpdatePostPayload>,
) -> Result<Json<Post>, AppError> {
    update_payload.validate()?;

    let mut conn = state.db_pool.get().expect("Failed to get a connection");

    let updated_post = tokio::task::spawn_blocking(move || {
        use crate::schema::posts::dsl::*;

        let post_to_update = posts.find(post_id).select(Post::as_select()).first(&mut conn)?;

        if post_to_update.user_id != claims.sub {
            return Err(AppError::Forbidden);
        }

        Ok(diesel::update(posts.find(post_id))
            .set(&update_payload)
            .returning(Post::as_returning())
            .get_result(&mut conn)?)
    })
    .await
    .unwrap()?;

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
    State(state): State<AppState>,
    claims: Claims,
    axum::extract::Path(post_id): axum::extract::Path<i32>,
) -> Result<StatusCode, AppError> {
    let mut conn = state.db_pool.get().expect("Failed to get a connection");

    let num_deleted = tokio::task::spawn_blocking(move || {
        use crate::schema::posts::dsl::*;
        use crate::schema::users::dsl as user_dsl;

        let post_to_delete = posts.find(post_id).select(Post::as_select()).first(&mut conn)?;
        let user = user_dsl::users.find(claims.sub).select(crate::models::user::User::as_select()).first(&mut conn)?;

        if post_to_delete.user_id != claims.sub && user.role != "admin" {
            return Err(AppError::Forbidden);
        }

        Ok(diesel::delete(posts.find(post_id)).execute(&mut conn)?)
    })
    .await
    .unwrap()?;

    if num_deleted == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
