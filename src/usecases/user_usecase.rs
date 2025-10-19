use std::sync::Arc;

use crate::{
    errors::AppError,
    models::user::{ChangePasswordRequest, CreateUser, UpdateUser, User},
    repositories::user_repository::UserRepository,
    security::{hash_password, verify_password},
};

pub struct UserUsecase {
    user_repo: Arc<UserRepository>,
}

impl UserUsecase {
    pub fn new(user_repo: Arc<UserRepository>) -> Self {
        UserUsecase { user_repo }
    }

    pub async fn create_user(&self, mut new_user: CreateUser) -> Result<User, AppError> {
        new_user.password = hash_password(new_user.password)
            .await
            .map_err(|e| AppError::InternalServerError(e))?;
        self.user_repo.create_user(new_user).await
    }

    pub async fn get_profile(&self, user_id: i32) -> Result<User, AppError> {
        self.user_repo.get_user_by_id(user_id).await
    }

    pub async fn update_profile(&self, user_id: i32, update_user: UpdateUser) -> Result<User, AppError> {
        self.user_repo.update_user(user_id, update_user).await
    }

    pub async fn change_password(&self, user_id: i32, password_data: ChangePasswordRequest) -> Result<(), AppError> {
        let user_password = self.user_repo.get_user_by_id(user_id).await?.password;

        let is_valid = verify_password(&user_password, &password_data.old_password)
            .map_err(|_| AppError::InternalServerError("Failed to verify password".to_string()))?;

        if !is_valid {
            return Err(AppError::Unauthorized);
        }

        let new_password_hash = hash_password(password_data.new_password)
            .await
            .map_err(|_| AppError::InternalServerError("Failed to hash password".to_string()))?;

        self.user_repo.change_password(user_id, new_password_hash).await?;

        Ok(())
    }

    pub async fn delete_profile(&self, user_id: i32) -> Result<(), AppError> {
        self.user_repo.delete_user(user_id).await?;
        Ok(())
    }

    pub async fn get_all_users(&self) -> Result<Vec<User>, AppError> {
        self.user_repo.get_all_users().await
    }

    pub async fn delete_user_by_id(&self, user_id: i32) -> Result<(), AppError> {
        let num_deleted = self.user_repo.delete_user(user_id).await?;

        if num_deleted == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }
}
