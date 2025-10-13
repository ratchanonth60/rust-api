use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager, Pool};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection() -> DbPool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    tracing::info!("Setting up database connection pool...");

    let manager = ConnectionManager::<PgConnection>::new(database_url);

    r2d2::Pool::builder()
        .max_size(10) // กำหนดจำนวน Connection สูงสุด
        .build(manager)
        .expect("Failed to create connection pool to Postgres")
}
