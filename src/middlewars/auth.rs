use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{request::Parts, Request},
    middleware::Next,
    response::Response,
};
use axum_extra::headers::{authorization::Bearer, Authorization, HeaderMapExt};

use crate::{errors::AppError, models::jwt::Claims, security::decode_token, state::AppState};

// Middleware function
pub async fn auth_guard<B>(
    State(state): State<AppState>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    // 1. ดึง Token จาก Header
    let token = req
        .headers()
        .typed_get::<Authorization<Bearer>>()
        .map(|auth| auth.token().to_owned())
        .ok_or(AppError::Unauthorized)?;

    // 2. ตรวจสอบความถูกต้องของ Token
    let claims =
        decode_token(&token, &state.config.jwt_secret).map_err(|_| AppError::Unauthorized)?;

    // 3. ถ้าถูกต้อง, เพิ่มข้อมูล claims เข้าไปใน request extensions
    // เพื่อให้ handler ปลายทางสามารถนำไปใช้ต่อได้
    req.extensions_mut().insert(claims);

    // 4. เรียก handler ตัวถัดไป
    Ok(next.run(req).await)
}

// สร้าง Extractor เพื่อให้ Handler ดึงข้อมูล Claims ได้ง่ายๆ
#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or(AppError::Unauthorized)
    }
}
