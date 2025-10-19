use crate::schema::categories;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;


#[derive(Queryable, Selectable, Serialize, Debug, Identifiable, ToSchema)]
#[diesel(table_name = categories)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

#[derive(Insertable, Deserialize, ToSchema, Validate)]
#[diesel(table_name = categories)]
pub struct CreateCategory {
    #[validate(length(min = 3))]
    pub name: String,
    #[validate(length(min = 3))]
    pub slug: String,
}
