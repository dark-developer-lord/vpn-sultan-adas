use dotenvy::dotenv;
use std::env;

pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub master_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv().ok();

        Ok(Config {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://vpn:vpn@localhost/vpn_service".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev-secret-key-change-in-production".to_string()),
            master_key: env::var("MASTER_KEY")
                .unwrap_or_else(|_| "dev-master-key-000000000000000000000000".to_string()),
        })
    }
}
