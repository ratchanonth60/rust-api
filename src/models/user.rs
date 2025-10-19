use crate::schema::users;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;


#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1))]
    pub username: String,
    #[validate(length(min = 1))]
    pub password: String,
}

// Struct สำหรับข้อมูล User ที่ดึงมาจาก Database
// สังเกตว่าเราไม่ใส่ field `password` เพราะไม่ควรส่งรหัสผ่านกลับไปให้ client
#[derive(Queryable, Selectable, Serialize, Debug, Identifiable, ToSchema)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String, // เก็บรหัสผ่านแบบ hash
    pub email: String,
    pub created_at: NaiveDateTime,
    pub role: String,
}

// Struct สำหรับรับข้อมูล JSON เข้ามาเพื่อสร้าง User ใหม่
#[derive(Insertable, Deserialize, ToSchema, Validate)]
#[diesel(table_name = users)]
pub struct CreateUser {
    #[validate(length(min = 3))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String, // รับรหัสผ่านเข้ามา
}

#[derive(Deserialize, AsChangeset, ToSchema)]
#[diesel(table_name = users)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub email: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}
