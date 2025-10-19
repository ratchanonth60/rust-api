use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};

use crate::{errors::AppError, models::jwt::Claims, state::AppState};
use std::sync::Arc;

pub async fn admin_guard(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let user = state.user_usecase.get_profile(claims.sub).await?;

    if user.role != "admin" {
        return Err(AppError::Forbidden);
    }

    Ok(next.run(req).await)
}
