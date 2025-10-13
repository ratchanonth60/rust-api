use crate::{handlers, state::AppState};
use axum::{
    routing::{get, post},
    Router,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health_handler::health_check,
        handlers::user_handler::create_user,
    ),
    components(
        schemas(crate::models::user::User, crate::models::user::CreateUser)
    ),
    tags((name = "API", description = "Rust API Endpoints"))
)]
struct ApiDoc;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(handlers::health_handler::health_check))
        .route("/users", post(handlers::user_handler::create_user))
        .with_state(state)
}
