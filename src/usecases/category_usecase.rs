use std::sync::Arc;

use crate::{
    errors::AppError,
    models::category::{Category, CreateCategory},
    repositories::category_repository::CategoryRepository,
};

pub struct CategoryUsecase {
    category_repo: Arc<CategoryRepository>,
}

impl CategoryUsecase {
    pub fn new(category_repo: Arc<CategoryRepository>) -> Self {
        CategoryUsecase { category_repo }
    }

    pub async fn create_category(&self, new_category: CreateCategory) -> Result<Category, AppError> {
        self.category_repo.create_category(new_category).await
    }

    pub async fn get_all_categories(&self) -> Result<Vec<Category>, AppError> {
        self.category_repo.get_all_categories().await
    }
}
