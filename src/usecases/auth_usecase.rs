use std::sync::Arc;

use crate::{
    errors::AppError,
    models::{
        jwt::{RefreshTokenPayload, TokenResponse},
        password_reset::{NewPasswordResetToken, PasswordResetToken},
        ForgotPasswordRequest, LoginRequest, ResetPasswordRequest, User,
    },
    repositories::user_repository::UserRepository,
    repositories::password_reset_token_repository::PasswordResetTokenRepository,
    security::{
        create_access_token, create_refresh_token, decode_token, hash_password, verify_password,
    },
    config::AppConfig,
};
use chrono::{Duration, Utc};
use rand::{distributions::Alphanumeric, Rng};

pub struct AuthUsecase {
    user_repo: Arc<UserRepository>,
    password_reset_token_repo: Arc<PasswordResetTokenRepository>,
    app_config: Arc<AppConfig>,
}

impl AuthUsecase {
    pub fn new(
        user_repo: Arc<UserRepository>,
        password_reset_token_repo: Arc<PasswordResetTokenRepository>,
        app_config: Arc<AppConfig>,
    ) -> Self {
        AuthUsecase {
            user_repo,
            password_reset_token_repo,
            app_config,
        }
    }

    pub async fn login(
        &self,
        login_user: LoginRequest,
    ) -> Result<TokenResponse, AppError> {
        let user_with_password = self.user_repo.get_user_by_username(login_user.username.clone()).await?;

        if !verify_password(&user_with_password.password, &login_user.password).unwrap_or(false) {
            return Err(AppError::Unauthorized);
        }

        let user = User {
            id: user_with_password.id,
            username: user_with_password.username,
            email: user_with_password.email,
            password: "".to_string(),
            created_at: user_with_password.created_at,
            role: user_with_password.role,
        };

        let access_token = create_access_token(&user, &self.app_config.jwt_secret)
            .map_err(|_| AppError::InternalServerError("Failed to create JWT".to_string()))?;
        let refresh_token = create_refresh_token(&user, &self.app_config.jwt_refresh_secret)
            .map_err(|_| AppError::InternalServerError("Failed to create JWT".to_string()))?;

        Ok(TokenResponse {
            access_token,
            refresh_token: refresh_token.clone(),
        })
    }

    pub async fn refresh_access_token(
        &self,
        payload: RefreshTokenPayload,
    ) -> Result<String, AppError> {
        let claims = decode_token(&payload.refresh_token, &self.app_config.jwt_refresh_secret)
            .map_err(|_| AppError::Unauthorized)?;

        let user = self.user_repo.get_user_by_id(claims.sub).await?;

        let new_access_token = create_access_token(&user, &self.app_config.jwt_secret).map_err(| _ |
            AppError::InternalServerError("Failed to create new access token".to_string())
        )?;

        Ok(new_access_token)
    }

    pub async fn forgot_password(
        &self,
        forgot_password_request: ForgotPasswordRequest,
    ) -> Result<(), AppError> {
        let user_email = forgot_password_request.email;

        // Find user by email
        let _user: User = self.user_repo.get_user_by_email(user_email.clone()).await?;

        // Generate a random token
        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        // Store the token in the database
        self.password_reset_token_repo.insert_or_update_token(NewPasswordResetToken {
            email: user_email.clone(),
            token: token.clone(),
        }).await?;

        // In a real application, you would send an email here.
        // For this example, we'll just log it.
        println!("Password reset token for {}: {}", user_email, token);

        Ok(())
    }

    pub async fn reset_password(
        &self,
        reset_password_request: ResetPasswordRequest,
    ) -> Result<(), AppError> {
        let request_token = reset_password_request.token;

        // Find the token in the database
        let reset_token: PasswordResetToken = self.password_reset_token_repo.find_token_by_token(request_token.clone()).await?;

        // Check if the token is expired (e.g., 1 hour)
        if Utc::now().naive_utc() - reset_token.created_at > Duration::hours(1) {
            self.password_reset_token_repo.delete_token(reset_token.token.clone()).await?;
            return Err(AppError::BadRequest("Invalid or expired token".to_string()));
        }

        // Hash the new password
        let new_password_hash = hash_password(reset_password_request.new_password)
            .await
            .map_err(|e| AppError::InternalServerError(e))?;

        // Update the user's password
        let user_to_update = self.user_repo.get_user_by_email(reset_token.email.clone()).await?;
        self.user_repo.change_password(user_to_update.id, new_password_hash).await?;

        // Delete the reset token
        self.password_reset_token_repo.delete_token(reset_token.token.clone()).await?;

        Ok(())
    }
}