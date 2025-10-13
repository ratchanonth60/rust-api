use axum::{http::StatusCode, Json};
use serde_json::{json, Value};

/// Handler for the health check endpoint.
/// Returns a 200 OK status and a JSON body indicating the service is healthy.
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = inline(serde_json::Value))
    )
)]
pub async fn health_check() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}
