use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use crate::schema::categories::dsl::*;
use crate::models::category::{Category, CreateCategory};
use crate::errors::AppError;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub struct CategoryRepository {
    pool: DbPool,
}

impl CategoryRepository {
    pub fn new(pool: DbPool) -> Self {
        CategoryRepository { pool }
    }

    pub async fn create_category(&self, new_category: CreateCategory) -> Result<Category, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            Ok(diesel::insert_into(categories)
                .values(&new_category)
                .returning(Category::as_returning())
                .get_result(&mut conn)?)
        })
        .await?
    }

    pub async fn get_all_categories(&self) -> Result<Vec<Category>, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            Ok(categories.select(Category::as_select()).load(&mut conn)?)
        })
        .await?
    }
}
