pub mod apple_auth;
pub mod chain;
pub mod email;
pub mod google_auth;
pub mod onramp;
pub mod prembly;
pub mod push;
pub mod sumsub;
pub mod wallet;

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
    pub prembly_secret_key: String,
    pub prembly_base_url: String,

    // V2 fields (§3.3, §13.7).
    pub privy_app_id: String,
    pub privy_app_secret: String,
    pub privy_jwks_url: String,
    pub alchemy_rpc_url: String,
    pub alchemy_webhook_secret: String,
    pub base_usdc_contract: String,          // canonical native USDC on Base
    pub escrow_wallet_address: String,       // §11.2 Option A platform wallet
    pub escrow_halt: bool,                   // §13.7 emergency circuit-breaker flag
    pub moonpay_api_key: String,
    pub moonpay_webhook_secret: String,
    pub yellow_card_api_key: String,
    pub sumsub_app_token: String,
    pub sumsub_secret_key: String,
    pub sumsub_webhook_secret: String,
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
            prembly_secret_key: std::env::var("PREMBLY_SECRET_KEY").unwrap_or_default(),
            prembly_base_url: std::env::var("PREMBLY_BASE_URL")
                .unwrap_or_else(|_| "https://api.prembly.com".to_string()),

            // V2 fields — all blank-by-default so Week 0 boots without
            // them. Each becomes required as its feature wires up.
            privy_app_id: std::env::var("PRIVY_APP_ID").unwrap_or_default(),
            privy_app_secret: std::env::var("PRIVY_APP_SECRET").unwrap_or_default(),
            privy_jwks_url: std::env::var("PRIVY_JWKS_URL")
                .unwrap_or_else(|_| "https://auth.privy.io/api/v1/apps/_/jwks.json".to_string()),
            alchemy_rpc_url: std::env::var("ALCHEMY_RPC_URL").unwrap_or_default(),
            alchemy_webhook_secret: std::env::var("ALCHEMY_WEBHOOK_SECRET").unwrap_or_default(),
            // Canonical native USDC on Base mainnet.
            // https://docs.base.org/base-contracts (chainId 8453)
            base_usdc_contract: std::env::var("BASE_USDC_CONTRACT")
                .unwrap_or_else(|_| "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string()),
            escrow_wallet_address: std::env::var("ESCROW_WALLET_ADDRESS").unwrap_or_default(),
            escrow_halt: std::env::var("ESCROW_HALT")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            moonpay_api_key: std::env::var("MOONPAY_API_KEY").unwrap_or_default(),
            moonpay_webhook_secret: std::env::var("MOONPAY_WEBHOOK_SECRET").unwrap_or_default(),
            yellow_card_api_key: std::env::var("YELLOW_CARD_API_KEY").unwrap_or_default(),
            sumsub_app_token: std::env::var("SUMSUB_APP_TOKEN").unwrap_or_default(),
            sumsub_secret_key: std::env::var("SUMSUB_SECRET_KEY").unwrap_or_default(),
            sumsub_webhook_secret: std::env::var("SUMSUB_WEBHOOK_SECRET").unwrap_or_default(),
        }
    }
}
