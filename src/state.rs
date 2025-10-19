use crate::{config::AppConfig, db::connect::DbPool};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    #[allow(dead_code)]
    pub config: AppConfig,
    pub rate_limiter: crate::middlewars::rate_limit::RateLimiter,
}
