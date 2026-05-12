//! Thin wrapper around Prembly's Identitypass REST API.
//!
//! Backend-proxy architecture: the `PREMBLY_SECRET_KEY` lives on the
//! server and never reaches the mobile client. Each verification call
//! returns both a typed view of the relevant fields and the raw JSON
//! response — we persist the raw body in JSONB columns for audit.
//!
//! Sandbox-only endpoints are noted on each method. Insurance NIID
//! lookup isn't enabled on unverified accounts; production access
//! needs a Nigerian CAC certificate.
//!
//! Sample sandbox responses (from probing on 2026-05-01):
//!   - DL: requires `first_name`, `last_name`, `number`, `dob`.
//!   - Plate: returns `vehicle_name`, `vehicle_color`, `vehicle_number`
//!     — but NOT the registered owner's name. Owner-mismatch detection
//!     is therefore self-declared on the client.

use reqwest::Client;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum PremblyError {
    #[error("PREMBLY_SECRET_KEY not configured")]
    NotConfigured,
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Prembly returned {status}: {body}")]
    BadResponse {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("Failed to parse Prembly response: {0}")]
    Parse(#[from] serde_json::Error),
}

/// Result of a verification call. `verified` reflects Prembly's top-
/// level `status: true` boolean — caller decides what to do with it.
/// `raw` is the full response body for audit storage.
pub struct VerificationOutcome {
    pub verified: bool,
    pub raw: Value,
}

#[derive(Serialize)]
struct DriversLicensePayload<'a> {
    number: &'a str,
    first_name: &'a str,
    last_name: &'a str,
    dob: String,
}

#[derive(Serialize)]
struct VehiclePlatePayload<'a> {
    vehicle_number: &'a str,
}

pub struct PremblyClient {
    secret_key: String,
    base_url: String,
    http: Client,
}

impl PremblyClient {
    pub fn new(secret_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            secret_key: secret_key.into(),
            base_url: base_url.into(),
            http: Client::new(),
        }
    }

    /// True if a secret key has been configured. Endpoints short-
    /// circuit with `NotConfigured` otherwise so dev runs without
    /// `PREMBLY_SECRET_KEY` don't crash.
    pub fn is_configured(&self) -> bool {
        !self.secret_key.is_empty()
    }

    /// Verify a Nigerian driver's licence against FRSC.
    ///
    /// Splits the host's `legal_full_name` into first/last on the
    /// caller side; we don't try to do that here because the caller
    /// already has the canonical name on the profile row.
    pub async fn verify_drivers_license(
        &self,
        first_name: &str,
        last_name: &str,
        number: &str,
        dob: chrono::NaiveDate,
    ) -> Result<VerificationOutcome, PremblyError> {
        if !self.is_configured() {
            return Err(PremblyError::NotConfigured);
        }
        let url = format!("{}/identitypass/verification/drivers_license", self.base_url);
        let body = DriversLicensePayload {
            number,
            first_name,
            last_name,
            dob: dob.format("%Y-%m-%d").to_string(),
        };
        let resp = self
            .http
            .post(&url)
            .header("x-api-key", &self.secret_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;
        self.parse(resp).await
    }

    /// Verify a Nigerian vehicle plate against FRSC.
    ///
    /// Plate must follow Prembly's "letters then numbers, min 8 chars"
    /// rule — strip any dashes/spaces before calling.
    pub async fn verify_vehicle_plate(
        &self,
        plate: &str,
    ) -> Result<VerificationOutcome, PremblyError> {
        if !self.is_configured() {
            return Err(PremblyError::NotConfigured);
        }
        let url = format!("{}/identitypass/verification/vehicle", self.base_url);
        let body = VehiclePlatePayload {
            vehicle_number: plate,
        };
        let resp = self
            .http
            .post(&url)
            .header("x-api-key", &self.secret_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;
        self.parse(resp).await
    }

    async fn parse(
        &self,
        resp: reqwest::Response,
    ) -> Result<VerificationOutcome, PremblyError> {
        let status = resp.status();
        let bytes = resp.bytes().await?;
        let raw: Value = serde_json::from_slice(&bytes).unwrap_or_else(|_| {
            // If Prembly returned non-JSON (HTML 404, etc.) keep the
            // textual body inside a JSON object so the audit row stays
            // valid JSONB.
            serde_json::json!({ "raw_text": String::from_utf8_lossy(&bytes).to_string() })
        });

        if !status.is_success() {
            // Some 4xx still come with a structured body (validation
            // errors). Surface them as a typed error rather than
            // swallowing — handler can choose what to render.
            let body = raw.to_string();
            return Err(PremblyError::BadResponse { status, body });
        }

        let verified = raw
            .get("status")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        Ok(VerificationOutcome { verified, raw })
    }
}
