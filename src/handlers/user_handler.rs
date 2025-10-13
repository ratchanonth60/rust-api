use crate::{
    db::connect::DbPool,
    errors::AppError,
    models::{CreateUser, User}, // Import model ที่เราสร้าง
};
use axum::{extract::State, http::StatusCode, Json};
use diesel::prelude::*;

// Handler สำหรับ POST /users
pub async fn create_user(
    State(pool): State<DbPool>,
    Json(new_user): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let mut conn = pool.get().expect("Failed to get a connection from pool");

    // ใน Production จริงๆ ควรจะ hash รหัสผ่านก่อนบันทึก
    // let hashed_password = hash_password(new_user.password);
    // new_user.password = hashed_password;

    let created_user = tokio::task::spawn_blocking(move || {
        use crate::schema::users::dsl::*;

        diesel::insert_into(users)
            .values(&new_user)
            // เราใช้ returning(User::as_returning()) เพื่อให้ Diesel ส่งข้อมูล user
            // ที่เพิ่งสร้างเสร็จกลับมา (โดยไม่มี field password ตามที่เรากำหนดใน struct User)
            .returning(User::as_returning())
            .get_result(&mut conn)
    })
    .await
    .unwrap()?; // Unwrap จาก JoinError, ? จาก diesel::result::Error

    Ok((StatusCode::CREATED, Json(created_user)))
}

