use crate::schema::password_reset_tokens;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Debug, Identifiable)]
#[diesel(table_name = password_reset_tokens)]
#[diesel(primary_key(email))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PasswordResetToken {
    pub email: String,
    pub token: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = password_reset_tokens)]
pub struct NewPasswordResetToken {
    pub email: String,
    pub token: String,
}
