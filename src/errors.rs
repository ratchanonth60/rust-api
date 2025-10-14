use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

// Define our application's error types
#[derive(Debug)] // Add Debug for better logging
pub enum AppError {
    DatabaseError(diesel::result::Error),
    NotFound,
    InternalServerError(String),
    Unauthorized,
}

// Allow converting from diesel::result::Error into our AppError
impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => AppError::NotFound,
            _ => AppError::DatabaseError(err),
        }
    }
}

// Define how to convert our AppError into a client-facing HTTP response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DatabaseError(db_err) => {
                // For production, log the detailed error but send a generic message
                tracing::error!("Database error: {:?}", db_err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal database error occurred".to_string(),
                )
            }
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized access".to_string()),
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "The requested resource was not found".to_string(),
            ),
            // ✅ Handle the new variant
            AppError::InternalServerError(msg) => {
                tracing::error!("Internal server error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An unexpected internal error occurred".to_string(),
                )
            }
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
