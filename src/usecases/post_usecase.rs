use std::sync::Arc;

use crate::{
    errors::AppError,
    models::{
        post::{CreatePostPayload, Post, UpdatePostPayload},
        pagination::Paginated,
    },
    repositories::post_repository::PostRepository,
    repositories::user_repository::UserRepository,
};

pub struct PostUsecase {
    post_repo: Arc<PostRepository>,
    user_repo: Arc<UserRepository>,
}

impl PostUsecase {
    pub fn new(post_repo: Arc<PostRepository>, user_repo: Arc<UserRepository>) -> Self {
        PostUsecase { post_repo, user_repo }
    }

    pub async fn create_post(&self, new_post: CreatePostPayload, user_id: i32) -> Result<Post, AppError> {
        self.post_repo.create_post(new_post, user_id).await
    }

    pub async fn get_posts(&self, page: i64, per_page: i64) -> Result<Paginated<Post>, AppError> {
        let (posts, total) = self.post_repo.get_posts(per_page, (page - 1) * per_page).await?;
        let total_pages = (total as f64 / per_page as f64).ceil() as i64;
        Ok(Paginated {
            items: posts,
            total_pages,
            page,
            per_page,
        })
    }

    pub async fn get_post_by_id(&self, post_id: i32) -> Result<Post, AppError> {
        self.post_repo.get_post_by_id(post_id).await
    }

    pub async fn get_posts_by_category(&self, slug_path: String) -> Result<Vec<Post>, AppError> {
        self.post_repo.get_posts_by_category(slug_path).await
    }

    pub async fn update_post(&self, post_id: i32, update_payload: UpdatePostPayload, claims_sub: i32) -> Result<Post, AppError> {
        let post_to_update = self.post_repo.get_post_by_id(post_id).await?;

        if post_to_update.user_id != claims_sub {
            return Err(AppError::Forbidden);
        }

        self.post_repo.update_post(post_id, update_payload).await
    }

    pub async fn delete_post(&self, post_id: i32, claims_sub: i32) -> Result<usize, AppError> {
        let post_to_delete = self.post_repo.get_post_by_id(post_id).await?;
        let user = self.user_repo.get_user_by_id(claims_sub).await?;

        if post_to_delete.user_id != claims_sub && user.role != "admin" {
            return Err(AppError::Forbidden);
        }

        let num_deleted = self.post_repo.delete_post(post_id).await?;

        if num_deleted == 0 {
            return Err(AppError::NotFound);
        }
        Ok(num_deleted)
    }
}
