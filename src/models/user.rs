use crate::schema::users;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

// Struct สำหรับข้อมูล User ที่ดึงมาจาก Database
// สังเกตว่าเราไม่ใส่ field `password` เพราะไม่ควรส่งรหัสผ่านกลับไปให้ client
#[derive(Queryable, Selectable, Serialize, Debug, Identifiable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: NaiveDateTime,
}

// Struct สำหรับรับข้อมูล JSON เข้ามาเพื่อสร้าง User ใหม่
#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String, // รับรหัสผ่านเข้ามา
}
