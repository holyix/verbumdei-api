use std::env;

pub struct AppConfig {
    host: String,
    port: u16,
    pub mongo_uri: String,
    pub mongo_db: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let host = env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("API_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080);

        let mongo_uri =
            env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        let mongo_db = env::var("MONGO_DB").unwrap_or_else(|_| "verbumdei".to_string());

        Self {
            host,
            port,
            mongo_uri,
            mongo_db,
        }
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
