#[derive(Clone)]
pub struct AppConfig {
    pub server_address: String,
    pub server_port: u16,
    pub master_key: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            server_address: std::env::var("SERVER_ADDRESS").unwrap_or_else(|_| "localhost".to_owned()),
            server_port: std::env::var("SERVER_PORT").expect("SERVER_PORT must be set").parse().expect("SERVER_PORT must be a valid u16"),
            master_key: std::env::var("MASTER_KEY").expect("MASTER_KEY must be set"),
        }
    }
}