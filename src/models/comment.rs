use crate::schema::comments;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Queryable, Selectable, Serialize, Debug, Identifiable, Associations, ToSchema)]
#[diesel(belongs_to(super::user::User))]
#[diesel(belongs_to(super::post::Post))]
#[diesel(table_name = comments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Comment {
    pub id: i32,
    pub content: String,
    pub user_id: i32,
    pub post_id: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize, ToSchema, Validate)]
#[diesel(table_name = comments)]
pub struct CreateCommentPayload {
    #[validate(length(min = 1))]
    pub content: String,
}

#[derive(Serialize, ToSchema)]
pub struct CommentResponse {
    pub id: i32,
    pub content: String,
    pub user_id: i32,
    pub post_id: i32,
    pub created_at: NaiveDateTime,
}
