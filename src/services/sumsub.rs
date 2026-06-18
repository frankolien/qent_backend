//! Sumsub KYC integration.
//!
//! V2 §3.3, §6.7, §10.1 — replaces Prembly as the identity provider.
//! Two operations:
//!   1. Create or fetch an applicant for a Qent user, then mint a
//!      short-lived WebSDK access token. The mobile client opens
//!      the inline Sumsub flow with that token (no external browser
//!      handoff).
//!   2. Verify Sumsub's outbound webhook signature (HMAC over the
//!      raw body) and turn the decision payload into a KYC tier
//!      update.
//!
//! Per-country Sumsub flow IDs are stored in `countries.sumsub_level`
//! (migration 024). NG uses a flow that adds the NIN add-on; other
//! markets use a standard ID + selfie + liveness.
//!
//! Stub for Week 0; live wiring lands in Week 1 (the tier_0→tier_1
//! gate) and Week 3 (the listing-flow tier_2 gate).

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum SumsubError {
    #[error("Sumsub credentials not configured")]
    NotConfigured,
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Sumsub returned {status}: {body}")]
    BadResponse {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("webhook signature invalid")]
    InvalidSignature,
    #[error("unknown applicant id")]
    UnknownApplicant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub token: String,
    pub user_id: String,
    pub level_name: String,
    pub expires_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone)]
pub enum SumsubDecision {
    Approved,
    Rejected { reason: String, retryable: bool },
    Pending,
}

#[derive(Clone)]
pub struct SumsubClient {
    app_token: String,
    secret_key: String,
    webhook_secret: String,
    http: reqwest::Client,
}

impl SumsubClient {
    pub fn new(app_token: String, secret_key: String, webhook_secret: String) -> Self {
        Self {
            app_token,
            secret_key,
            webhook_secret,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("reqwest client"),
        }
    }

    /// Find-or-create a Sumsub applicant for this Qent user,
    /// scoped to the per-country flow id. The returned applicant
    /// id should be persisted to `users.sumsub_applicant_id`.
    pub async fn ensure_applicant(
        &self,
        _qent_user_id: &str,
        _level_name: &str,
    ) -> Result<String, SumsubError> {
        todo!("Week 1 — POST /resources/applicants?levelName=...")
    }

    /// Mint a short-lived WebSDK access token for the mobile
    /// client. Each token is single-use, scoped to one user, and
    /// expires within minutes per Sumsub's recommendation.
    pub async fn create_access_token(
        &self,
        _qent_user_id: &str,
        _level_name: &str,
    ) -> Result<AccessToken, SumsubError> {
        todo!("Week 1 — POST /resources/accessTokens with signature")
    }

    /// Verify an inbound webhook signature against the raw body,
    /// per §4.3 (signature-first discipline).
    pub fn verify_webhook(
        &self,
        _signature_header: &str,
        _raw_body: &[u8],
    ) -> Result<(), SumsubError> {
        todo!("Week 1 — HMAC-SHA256 over raw body, constant-time compare")
    }

    /// Map a Sumsub webhook payload to a `SumsubDecision`.
    pub fn parse_decision(&self, _raw_body: &[u8]) -> Result<SumsubDecision, SumsubError> {
        todo!("Week 1 — parse review.reviewStatus + review.reviewAnswer")
    }
}
