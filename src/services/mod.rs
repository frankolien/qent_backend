pub mod apple_auth;
pub mod email;
pub mod google_auth;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub paystack_secret_key: String,
    pub resend_api_key: String,
    pub app_url: String,
    pub host: String,
    pub port: u16,
    pub apple_bundle_id: String,
    pub google_client_ids: Vec<String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let google_client_ids = std::env::var("GOOGLE_CLIENT_IDS")
            .or_else(|_| std::env::var("GOOGLE_CLIENT_ID"))
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .collect();

        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            paystack_secret_key: std::env::var("PAYSTACK_SECRET_KEY")
                .expect("PAYSTACK_SECRET_KEY must be set"),
            resend_api_key: std::env::var("RESEND_API_KEY").unwrap_or_default(),
            app_url: std::env::var("APP_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            host: std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            apple_bundle_id: std::env::var("APPLE_BUNDLE_ID").unwrap_or_default(),
            google_client_ids,
        }
    }
}
