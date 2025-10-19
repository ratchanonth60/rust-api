use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use validator::ValidationErrors;

// Define our application's error types
#[derive(Debug)] // Add Debug for better logging
pub enum AppError {
    DatabaseError(diesel::result::Error),
    NotFound,
    InternalServerError(String),
    Unauthorized,
    Forbidden,
    BadRequest(String),
    InvalidInput(ValidationErrors),
    DuplicateEntry,
}

// Allow converting from diesel::result::Error into our AppError
impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => AppError::NotFound,
            diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _) => {
                AppError::DuplicateEntry
            }
            _ => AppError::DatabaseError(err),
        }
    }
}

impl From<ValidationErrors> for AppError {
    fn from(errors: ValidationErrors) -> Self {
        AppError::InvalidInput(errors)
    }
}

impl From<tokio::task::JoinError> for AppError {
    fn from(err: tokio::task::JoinError) -> Self {
        AppError::InternalServerError(format!("Task join error: {}", err))
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
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden access".to_string()),
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
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::InvalidInput(errors) => {
                let messages = errors
                    .field_errors()
                    .into_iter()
                    .map(|(field, errors)| {
                        let messages = errors
                            .iter()
                            .map(|e| e.message.as_ref().unwrap().to_string())
                            .collect::<Vec<_>>();
                        (field, messages)
                    })
                    .collect::<std::collections::HashMap<_, _>>();
                return (StatusCode::BAD_REQUEST, Json(json!({ "errors": messages }))).into_response();
            }
            AppError::DuplicateEntry => (StatusCode::CONFLICT, "Email or username already exists".to_string()),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
