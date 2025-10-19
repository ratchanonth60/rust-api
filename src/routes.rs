use crate::{handlers, middlewars, state::AppState};
use axum::{
    middleware,
    routing::{delete, get, patch, post, put},
    Router,
};
use http::Method;
use tower_http::{
    cors::{Any, CorsLayer},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use std::sync::Arc;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Auth
        handlers::auth_handler::login,
        handlers::auth_handler::refresh_access_token,
        handlers::auth_handler::forgot_password,
        handlers::auth_handler::reset_password,
        // Health
        handlers::health_handler::health_check,
        // User
        handlers::user_handler::create_user,
        handlers::user_handler::get_all_users,
        handlers::user_handler::get_profile,
        handlers::user_handler::update_profile,
        handlers::user_handler::change_password,
        handlers::user_handler::delete_profile,
        handlers::user_handler::delete_user_by_id,
        // Category
        handlers::category_handler::create_category,
        handlers::category_handler::get_categories,
        // Post
        handlers::post_handler::create_post,
        handlers::post_handler::get_posts,
        handlers::post_handler::get_post_by_id,
        handlers::post_handler::get_posts_by_category,
        handlers::post_handler::update_post,
        handlers::post_handler::delete_post,
        // Comment
        handlers::comment_handler::create_comment,
        handlers::comment_handler::get_comments_for_post,
        handlers::comment_handler::update_comment,
        handlers::comment_handler::delete_comment,
    ),
    components(
        schemas(
            // User
            crate::models::user::User,
            crate::models::user::CreateUser,
            crate::models::user::LoginRequest,
            crate::models::user::UpdateUser,
            crate::models::user::ChangePasswordRequest,
            crate::models::user::ForgotPasswordRequest,
            crate::models::user::ResetPasswordRequest,
            // Category
            crate::models::category::Category,
            crate::models::category::CreateCategory,
            // Post
            crate::models::post::Post,
            crate::models::post::CreatePostPayload,
            crate::models::post::UpdatePostPayload,
            crate::models::post::PostResponse,
            // Comment
            crate::models::comment::Comment,
            crate::models::comment::CreateCommentPayload,
            crate::models::comment::CommentResponse,
            // Pagination
            crate::models::pagination::Paginated<crate::models::post::Post>,
        )
    ),
    tags((name = "API", description = "Rust API Endpoints"))
)]
struct ApiDoc;

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::PUT, Method::DELETE])
        .allow_origin(Any);

    let admin_routes = Router::<Arc<AppState>>::new()
        .route("/users", get::<_, _, Arc<AppState>>(handlers::user_handler::get_all_users))
        .route("/users/:id", delete::<_, _, Arc<AppState>>(handlers::user_handler::delete_user_by_id))
        .route("/categories", post::<_, _, Arc<AppState>>(handlers::category_handler::create_category))
        .with_state(state.clone())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            middlewars::admin::admin_guard,
        ));

    let protected_routes = Router::<Arc<AppState>>::new()
        .route("/profile", get::<_, _, Arc<AppState>>(handlers::user_handler::get_profile))
        .route("/profile", patch::<_, _, Arc<AppState>>(handlers::user_handler::update_profile))
        .route("/profile", delete::<_, _, Arc<AppState>>(handlers::user_handler::delete_profile))
        .route("/profile/password", put::<_, _, Arc<AppState>>(handlers::user_handler::change_password))
        .route("/posts", post::<_, _, Arc<AppState>>(handlers::post_handler::create_post))
        .route("/posts/:id", patch::<_, _, Arc<AppState>>(handlers::post_handler::update_post))
        .route("/posts/:id", delete::<_, _, Arc<AppState>>(handlers::post_handler::delete_post))
        .route("/posts/:id/comments", post::<_, _, Arc<AppState>>(handlers::comment_handler::create_comment))
        .route("/comments/:id", patch::<_, _, Arc<AppState>>(handlers::comment_handler::update_comment))
        .route("/comments/:id", delete::<_, _, Arc<AppState>>(handlers::comment_handler::delete_comment))
        .merge(admin_routes)
        .with_state(state.clone())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            middlewars::auth::auth_guard,
        ));

    let rate_limited_auth_routes = Router::<Arc<AppState>>::new()
        .route("/login", post::<_, _, Arc<AppState>>(handlers::auth_handler::login))
        .route("/forgot-password", post::<_, _, Arc<AppState>>(handlers::auth_handler::forgot_password))
        .with_state(state.clone())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            middlewars::rate_limit::rate_limit_middleware,
        ));

    let public_routes = Router::<Arc<AppState>>::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/", get::<_, _, Arc<AppState>>(handlers::health_handler::health_check))
        .route(
            "/refresh",
            post::<_, _, Arc<AppState>>(handlers::auth_handler::refresh_access_token),
        )
        .route("/reset-password", post::<_, _, Arc<AppState>>(handlers::auth_handler::reset_password))
        .route("/users", post::<_, _, Arc<AppState>>(handlers::user_handler::create_user))
        .route("/categories", get::<_, _, Arc<AppState>>(handlers::category_handler::get_categories))
        .route("/posts", get::<_, _, Arc<AppState>>(handlers::post_handler::get_posts))
        .route("/posts/:id", get::<_, _, Arc<AppState>>(handlers::post_handler::get_post_by_id))
        .route("/categories/:slug/posts", get::<_, _, Arc<AppState>>(handlers::post_handler::get_posts_by_category))
        .route("/posts/:id/comments", get::<_, _, Arc<AppState>>(handlers::comment_handler::get_comments_for_post))
        .merge(rate_limited_auth_routes)
        .with_state(state.clone());

    Router::<Arc<AppState>>::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state)
        .layer(cors)
}