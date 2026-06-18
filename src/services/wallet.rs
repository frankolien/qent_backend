//! Privy embedded-wallet operations.
//!
//! Per the V2 design doc §3.3 and §6.5, every user gets an
//! app-controlled wallet on Base provisioned through Privy during
//! Phase 4 of onboarding. This module is the server-side wrapper
//! around the Privy REST API: create wallet for user, look up by
//! Privy user id, instruct programmatic transfers from the escrow
//! wallet at booking-completion time (§4.1 step 20).
//!
//! All Privy API calls go through here. The `PRIVY_APP_SECRET`
//! never leaves this module; handler code calls into typed methods.
//!
//! Stub for Week 0 of §12.1 — signatures fixed, real HTTP wiring
//! lands in Week 1.

use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    #[error("Privy credentials not configured")]
    NotConfigured,
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Privy returned {status}: {body}")]
    BadResponse {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("Privy session token failed JWKS verification")]
    InvalidSession,
}

/// One Privy-managed embedded wallet on Base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivyWallet {
    pub privy_wallet_id: String,
    pub address: String,
    pub chain: String,
}

/// The fields we keep from a verified Privy session — used by
/// `auth::privy_session` to find-or-create a `users` row.
#[derive(Debug, Clone)]
pub struct VerifiedPrivySession {
    pub privy_user_id: String,
    pub email: Option<String>,
    pub auth_provider: String, // "google" | "apple" | "email"
}

#[derive(Clone)]
pub struct WalletClient {
    http: Client,
    app_id: String,
    app_secret: String,
    jwks_url: String,
}

impl WalletClient {
    pub fn new(app_id: String, app_secret: String, jwks_url: String) -> Self {
        Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("reqwest client"),
            app_id,
            app_secret,
            jwks_url,
        }
    }

    /// Validate a Privy session token from the mobile client and
    /// return the verified identity. Called by
    /// `POST /api/auth/privy-session` (§10.1).
    pub async fn verify_session(
        &self,
        _privy_token: &str,
    ) -> Result<VerifiedPrivySession, WalletError> {
        // §6.2: fetch JWKS, validate signature, extract sub + email.
        todo!("Week 1 — JWKS fetch + RS256 verify")
    }

    /// Create an app-controlled embedded wallet for a Privy user
    /// on Base. Called once per user during Phase 4 of onboarding.
    pub async fn create_wallet(&self, _privy_user_id: &str) -> Result<PrivyWallet, WalletError> {
        // POST {privy_base}/v1/users/{id}/wallets {chain: "base"}
        todo!("Week 1 — Privy create_wallet API call")
    }

    /// Look up a wallet by its Privy id (cached locally in the
    /// `wallets` table; this is only for repair / out-of-band cases).
    pub async fn fetch_wallet(&self, _privy_wallet_id: &str) -> Result<PrivyWallet, WalletError> {
        todo!("Week 1 — Privy fetch_wallet API call")
    }

    /// Instruct a transfer from the platform escrow wallet to a
    /// destination address. The §4.1 step 20 payout path calls into
    /// this; the §13.7 outflow circuit-breaker wraps it.
    pub async fn escrow_transfer(
        &self,
        _to_address: &str,
        _amount_usdc: rust_decimal::Decimal,
    ) -> Result<String /* tx_hash */, WalletError> {
        todo!("Week 2 — escrow transfer via Privy server API, gated by escrow_breaker")
    }
}
