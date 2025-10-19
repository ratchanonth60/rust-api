use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use crate::schema::comments::dsl::*;
use crate::models::comment::{Comment, CreateCommentPayload};
use crate::errors::AppError;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub struct CommentRepository {
    pool: DbPool,
}

impl CommentRepository {
    pub fn new(pool: DbPool) -> Self {
        CommentRepository { pool }
    }

    pub async fn create_comment(&self, new_comment: CreateCommentPayload, current_user_id: i32, current_post_id: i32) -> Result<Comment, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            let comment_data = (
                content.eq(&new_comment.content),
                crate::schema::comments::dsl::user_id.eq(current_user_id),
                crate::schema::comments::dsl::post_id.eq(current_post_id),
            );
            Ok(diesel::insert_into(comments)
                .values(comment_data)
                .returning(Comment::as_returning())
                .get_result(&mut conn)?)
        })
        .await?
    }

    pub async fn get_comments_for_post(&self, post_id_path: i32) -> Result<Vec<Comment>, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            Ok(comments.filter(post_id.eq(post_id_path)).select(Comment::as_select()).load(&mut conn)?)
        })
        .await?
    }

    pub async fn update_comment(&self, comment_id_path: i32, update_payload: CreateCommentPayload) -> Result<Comment, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            Ok(diesel::update(comments.find(comment_id_path))
                .set(content.eq(&update_payload.content))
                .returning(Comment::as_returning())
                .get_result(&mut conn)?)
        })
        .await?
    }

    pub async fn delete_comment(&self, comment_id_path: i32) -> Result<usize, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            Ok(diesel::delete(comments.find(comment_id_path)).execute(&mut conn)?)
        })
        .await?
    }

    pub async fn get_comment_by_id(&self, comment_id: i32) -> Result<Comment, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            Ok(comments.find(comment_id).select(Comment::as_select()).first(&mut conn)?)
        })
        .await?
    }
}
