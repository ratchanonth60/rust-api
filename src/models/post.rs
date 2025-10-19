use crate::schema::posts;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Queryable, Selectable, Serialize, Debug, Identifiable, Associations, ToSchema)]
#[diesel(belongs_to(super::user::User))]
#[diesel(belongs_to(super::category::Category))]
#[diesel(table_name = posts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub user_id: i32,
    pub category_id: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize, ToSchema, Validate)]
#[diesel(table_name = posts)]
pub struct CreatePostPayload {
    #[validate(length(min = 3))]
    pub title: String,
    #[validate(length(min = 3))]
    pub content: String,
    pub category_id: i32,
}

#[derive(Deserialize, AsChangeset, ToSchema, Validate)]
#[diesel(table_name = posts)]
pub struct UpdatePostPayload {
    #[validate(length(min = 3))]
    pub title: Option<String>,
    #[validate(length(min = 3))]
    pub content: Option<String>,
    pub category_id: Option<i32>,
}

#[derive(Serialize, ToSchema)]
pub struct PostResponse {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub user_id: i32,
    pub category_id: i32,
    pub created_at: NaiveDateTime,
}
