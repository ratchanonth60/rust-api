use std::env;

#[derive(Clone)] // Clone is needed to pass it to the app state
pub struct AppConfig {
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
}

impl AppConfig {
    /// Loads configuration from environment variables.
    /// Panics if any required variable is not set.
    pub fn from_env() -> Self {
        let server_host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .expect("PORT must be a valid number");

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        AppConfig {
            server_host,
            server_port,
            database_url,
        }
    }
}
