use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager, Pool};
use tracing::info;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection(database_url: &str) -> DbPool {
    info!("Setting up database connection pool...");

    let manager = ConnectionManager::<PgConnection>::new(database_url);

    r2d2::Pool::builder()
        .max_size(15) // You can configure pool size
        .build(manager)
        .expect("Failed to create connection pool to Postgres")
}
