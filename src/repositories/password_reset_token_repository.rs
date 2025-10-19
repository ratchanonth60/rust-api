use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use crate::schema::password_reset_tokens::dsl::*;
use crate::models::password_reset::{NewPasswordResetToken, PasswordResetToken};
use crate::errors::AppError;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub struct PasswordResetTokenRepository {
    pool: DbPool,
}

impl PasswordResetTokenRepository {
    pub fn new(pool: DbPool) -> Self {
        PasswordResetTokenRepository { pool }
    }

    pub async fn insert_or_update_token(&self, new_token: NewPasswordResetToken) -> Result<usize, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            Ok(diesel::insert_into(password_reset_tokens)
                .values(&new_token)
                .on_conflict(email)
                .do_update()
                .set(token.eq(&new_token.token))
                .execute(&mut conn)?)
        })
        .await?
    }

    pub async fn find_token_by_token(&self, token_str: String) -> Result<PasswordResetToken, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            Ok(password_reset_tokens
                .filter(token.eq(&token_str))
                .first(&mut conn)?)
        })
        .await?
    }

    pub async fn delete_token(&self, token_str: String) -> Result<usize, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            Ok(diesel::delete(password_reset_tokens.filter(token.eq(&token_str)))
                .execute(&mut conn)?)
        })
        .await?
    }
}
