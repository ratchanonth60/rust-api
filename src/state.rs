use crate::config::AppConfig;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    pub config: AppConfig,
    pub rate_limiter: crate::middlewars::rate_limit::RateLimiter,
    pub auth_usecase: Arc<crate::usecases::auth_usecase::AuthUsecase>,
    pub user_usecase: Arc<crate::usecases::user_usecase::UserUsecase>,
    pub post_usecase: Arc<crate::usecases::post_usecase::PostUsecase>,
    pub category_usecase: Arc<crate::usecases::category_usecase::CategoryUsecase>,
    pub comment_usecase: Arc<crate::usecases::comment_usecase::CommentUsecase>,
}
