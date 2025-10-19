use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};
use diesel::prelude::*;

use crate::{errors::AppError, models::jwt::Claims, models::user::User, state::AppState};

pub async fn admin_guard(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let mut conn = state.db_pool.get().expect("Failed to get a connection");
    let user = tokio::task::spawn_blocking(move || {
        use crate::schema::users::dsl::*;
        users
            .find(claims.sub)
            .select(User::as_select())
            .first(&mut conn)
    })
    .await
    .unwrap()?;

    if user.role != "admin" {
        return Err(AppError::Forbidden);
    }

    Ok(next.run(req).await)
}
