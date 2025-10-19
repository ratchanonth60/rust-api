use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total_pages: i64,
    pub page: i64,
    pub per_page: i64,
}
