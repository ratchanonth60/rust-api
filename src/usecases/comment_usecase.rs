use std::sync::Arc;

use crate::{
    errors::AppError,
    models::{
        comment::{Comment, CreateCommentPayload},
    },
    repositories::comment_repository::CommentRepository,
    repositories::user_repository::UserRepository,
};

pub struct CommentUsecase {
    comment_repo: Arc<CommentRepository>,
    user_repo: Arc<UserRepository>,
}

impl CommentUsecase {
    pub fn new(comment_repo: Arc<CommentRepository>, user_repo: Arc<UserRepository>) -> Self {
        CommentUsecase { comment_repo, user_repo }
    }

    pub async fn create_comment(&self, new_comment: CreateCommentPayload, user_id: i32, post_id: i32) -> Result<Comment, AppError> {
        self.comment_repo.create_comment(new_comment, user_id, post_id).await
    }

    pub async fn get_comments_for_post(&self, post_id: i32) -> Result<Vec<Comment>, AppError> {
        self.comment_repo.get_comments_for_post(post_id).await
    }

    pub async fn update_comment(&self, comment_id: i32, update_payload: CreateCommentPayload, claims_sub: i32) -> Result<Comment, AppError> {
        let comment_to_update = self.comment_repo.get_comment_by_id(comment_id).await?;

        if comment_to_update.user_id != claims_sub {
            return Err(AppError::Forbidden);
        }

        self.comment_repo.update_comment(comment_id, update_payload).await
    }

    pub async fn delete_comment(&self, comment_id: i32, claims_sub: i32) -> Result<usize, AppError> {
        let comment_to_delete = self.comment_repo.get_comment_by_id(comment_id).await?;
        let user = self.user_repo.get_user_by_id(claims_sub).await?;

        if comment_to_delete.user_id != claims_sub && user.role != "admin" {
            return Err(AppError::Forbidden);
        }

        let num_deleted = self.comment_repo.delete_comment(comment_id).await?;

        if num_deleted == 0 {
            return Err(AppError::NotFound);
        }
        Ok(num_deleted)
    }
}
