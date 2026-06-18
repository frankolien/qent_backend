//! On-ramp / off-ramp orchestration.
//!
//! V2 §4.5, §4.6, §10.1 — quote prices and generate hosted-flow
//! URLs for renters (card → USDC) and hosts (USDC → fiat). Partner
//! pool: MoonPay (everywhere), Yellow Card (NG corridor, faster +
//! thinner spread), Onramper (aggregator we may add later).
//!
//! The user-facing widget always opens in a partner-hosted page;
//! this module's job is the quoting + reconciliation glue, not the
//! UI surface.
//!
//! Stub for Week 0; MoonPay + Yellow Card integration lands in
//! Week 4.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum OnrampError {
    #[error("on-ramp partner not configured")]
    NotConfigured,
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("partner returned {status}: {body}")]
    BadResponse {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("no partner has a quote for this corridor")]
    NoQuote,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Direction {
    OnRamp,  // fiat → USDC
    OffRamp, // USDC → fiat
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub partner: String,           // "moonpay" | "yellow_card"
    pub direction: Direction,
    pub amount_usdc: Decimal,
    pub amount_fiat: Decimal,
    pub fiat_currency: String,     // "NGN" | "EUR" | ...
    pub quoted_rate: Decimal,      // fiat per USDC
    pub partner_fee_usdc: Decimal, // their cut
    pub our_spread_usdc: Decimal,  // our cut (§2.2)
    pub expires_at: chrono::NaiveDateTime,
    pub hosted_flow_url: Option<String>, // for hosted-widget partners
    pub quote_id: String,
}

#[derive(Clone)]
pub struct OnrampClient {
    moonpay_api_key: Option<String>,
    yellow_card_api_key: Option<String>,
    http: reqwest::Client,
}

impl OnrampClient {
    pub fn new(moonpay_api_key: Option<String>, yellow_card_api_key: Option<String>) -> Self {
        Self {
            moonpay_api_key,
            yellow_card_api_key,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("reqwest client"),
        }
    }

    /// Ask every configured partner for a quote on this corridor;
    /// return the one that gives the best rate-after-our-spread.
    /// Per §4.6, we default to whichever is best for the host /
    /// renter, not whichever is on our preferred-partner list.
    pub async fn best_quote(
        &self,
        _direction: Direction,
        _amount_usdc: Decimal,
        _fiat_currency: &str,
        _country: &str,
    ) -> Result<Quote, OnrampError> {
        todo!("Week 4 — fan out to partners, pick best rate")
    }

    /// Execute a previously-quoted off-ramp for a host. Returns
    /// the partner's reference for reconciliation.
    pub async fn execute_offramp(
        &self,
        _quote_id: &str,
        _destination_account: &str,
    ) -> Result<String, OnrampError> {
        todo!("Week 4 — partner-specific execute call")
    }
}
