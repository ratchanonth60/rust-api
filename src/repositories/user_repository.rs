use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use crate::schema::users::dsl::*;
use crate::models::user::{User, CreateUser, UpdateUser};
use crate::errors::AppError;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub struct UserRepository {
    pool: DbPool,
}

impl UserRepository {
    pub fn new(pool: DbPool) -> Self {
        UserRepository { pool }
    }

    pub async fn create_user(&self, new_user: CreateUser) -> Result<User, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            Ok(diesel::insert_into(users)
                .values(&new_user)
                .returning(User::as_returning())
                .get_result(&mut conn)?)
        })
        .await?
    }

    pub async fn get_user_by_id(&self, user_id: i32) -> Result<User, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            Ok(users.filter(id.eq(user_id)).select(User::as_select()).first(&mut conn)?)
        })
        .await?
    }

    pub async fn get_user_by_email(&self, user_email: String) -> Result<User, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            Ok(users.filter(email.eq(user_email)).select(User::as_select()).first(&mut conn)?)
        })
        .await?
    }

    pub async fn get_user_by_username(&self, user_username: String) -> Result<User, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            Ok(users.filter(username.eq(user_username)).select(User::as_select()).first(&mut conn)?)
        })
        .await?
    }

    pub async fn update_user(&self, user_id: i32, update_user: UpdateUser) -> Result<User, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            Ok(diesel::update(users.filter(id.eq(user_id)))
                .set(&update_user)
                .returning(User::as_returning())
                .get_result(&mut conn)?)
        })
        .await?
    }

    pub async fn change_password(&self, user_id: i32, new_password_hash: String) -> Result<usize, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            Ok(diesel::update(users.filter(id.eq(user_id)))
                .set(password.eq(new_password_hash))
                .execute(&mut conn)?)
        })
        .await?
    }

    pub async fn delete_user(&self, user_id: i32) -> Result<usize, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            Ok(diesel::delete(users.filter(id.eq(user_id))).execute(&mut conn)?)
        })
        .await?
    }

    pub async fn get_all_users(&self) -> Result<Vec<User>, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            Ok(users.select(User::as_select()).load(&mut conn)?)
        })
        .await?
    }
}
