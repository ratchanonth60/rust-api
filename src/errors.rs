use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

// กำหนด Error Type ของเรา
pub enum AppError {
    DatabaseError(diesel::result::Error),
    NotFound,
}

// ทำให้ AppError สามารถแปลงจาก diesel::result::Error ได้โดยตรง
// เพื่อให้เราสามารถใช้ `?` operator ได้ใน handler
impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => AppError::NotFound,
            _ => AppError::DatabaseError(err),
        }
    }
}

// แปลง AppError ของเราให้เป็น HTTP Response ที่ Axum เข้าใจ
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DatabaseError(db_err) => {
                // สำหรับ Production ควร log error จริงๆ ไว้ แต่ส่งข้อความทั่วไปให้ client
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal server error: {}", db_err),
                )
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
