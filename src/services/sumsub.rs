//! Sumsub KYC integration.
//!
//! V2 §3.3, §6.7, §10.1 — replaces Prembly as the identity provider.
//! Two responsibilities:
//!   1. Find-or-create an applicant for a Qent user, then mint a
//!      short-lived WebSDK access token. The mobile client opens the
//!      inline Sumsub flow with that token (no external browser
//!      handoff).
//!   2. Verify Sumsub's outbound webhook signature (HMAC over the
//!      raw body) and turn the decision payload into a KYC tier
//!      update.
//!
//! Per-country Sumsub flow IDs are stored in `countries.sumsub_level`
//! (migration 024). NG uses a flow that adds the NIN add-on; other
//! markets use a standard ID + selfie + liveness.

use chrono::{NaiveDateTime, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;

const SUMSUB_API_BASE: &str = "https://api.sumsub.com";

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
    #[error("payload malformed: {0}")]
    MalformedPayload(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AccessToken {
    pub token: String,
    pub user_id: String,
    pub level_name: String,
    pub expires_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub enum SumsubDecision {
    Approved {
        applicant_id: String,
        external_user_id: String,
    },
    Rejected {
        applicant_id: String,
        external_user_id: String,
        reason: String,
        retryable: bool,
    },
    Pending {
        applicant_id: String,
        external_user_id: String,
    },
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

    fn ensure_configured(&self) -> Result<(), SumsubError> {
        if self.app_token.is_empty() || self.secret_key.is_empty() {
            return Err(SumsubError::NotConfigured);
        }
        Ok(())
    }

    /// Sign a Sumsub-bound request per their docs: ts + METHOD + path
    /// (with query) + body, HMAC-SHA256 hex, app token + ts headers.
    fn sign(
        &self,
        method: &str,
        path_with_query: &str,
        body: &[u8],
    ) -> (String /* sig */, String /* ts */) {
        let ts = Utc::now().timestamp().to_string();
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC key");
        mac.update(ts.as_bytes());
        mac.update(method.as_bytes());
        mac.update(path_with_query.as_bytes());
        mac.update(body);
        let sig = hex::encode(mac.finalize().into_bytes());
        (sig, ts)
    }

    async fn send_signed(
        &self,
        method: reqwest::Method,
        path_with_query: &str,
        body_json: Option<&serde_json::Value>,
    ) -> Result<reqwest::Response, SumsubError> {
        let body_bytes = match body_json {
            Some(v) => serde_json::to_vec(v).unwrap_or_default(),
            None => Vec::new(),
        };
        let (sig, ts) = self.sign(method.as_str(), path_with_query, &body_bytes);

        let url = format!("{SUMSUB_API_BASE}{path_with_query}");
        let mut req = self
            .http
            .request(method, &url)
            .header("X-App-Token", &self.app_token)
            .header("X-App-Access-Sig", sig)
            .header("X-App-Access-Ts", ts);
        if body_json.is_some() {
            req = req
                .header("Content-Type", "application/json")
                .body(body_bytes);
        }
        Ok(req.send().await?)
    }

    /// Find-or-create a Sumsub applicant for this Qent user, scoped
    /// to the per-country flow id. The returned applicant id should
    /// be persisted to `users.sumsub_applicant_id`.
    pub async fn ensure_applicant(
        &self,
        qent_user_id: &str,
        level_name: &str,
    ) -> Result<String, SumsubError> {
        self.ensure_configured()?;

        // 1) Try GET by externalUserId. Sumsub returns 404 if absent.
        let lookup_path = format!("/resources/applicants/-;externalUserId={qent_user_id}");
        let resp = self
            .send_signed(reqwest::Method::GET, &lookup_path, None)
            .await?;

        if resp.status().is_success() {
            let payload: serde_json::Value = resp.json().await?;
            if let Some(id) = payload.get("id").and_then(|v| v.as_str()) {
                return Ok(id.to_string());
            }
        }

        // 2) Not found — create.
        let create_path = format!("/resources/applicants?levelName={level_name}");
        let body = serde_json::json!({ "externalUserId": qent_user_id });
        let resp = self
            .send_signed(reqwest::Method::POST, &create_path, Some(&body))
            .await?;

        let status = resp.status();
        let payload_text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(SumsubError::BadResponse {
                status,
                body: payload_text,
            });
        }

        let payload: serde_json::Value = serde_json::from_str(&payload_text)
            .map_err(|e| SumsubError::MalformedPayload(e.to_string()))?;
        let id = payload
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SumsubError::MalformedPayload("applicant response missing id".into()))?
            .to_string();
        Ok(id)
    }

    /// Mint a short-lived WebSDK access token for the mobile client.
    /// Each token is single-use, scoped to one user, and expires
    /// within minutes per Sumsub's recommendation.
    pub async fn create_access_token(
        &self,
        qent_user_id: &str,
        level_name: &str,
    ) -> Result<AccessToken, SumsubError> {
        self.ensure_configured()?;

        const TTL_SECS: i64 = 600;
        let path = format!(
            "/resources/accessTokens?userId={qent_user_id}&levelName={level_name}&ttlInSecs={TTL_SECS}",
        );
        let resp = self
            .send_signed(reqwest::Method::POST, &path, None)
            .await?;
        let status = resp.status();
        let payload_text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(SumsubError::BadResponse {
                status,
                body: payload_text,
            });
        }

        let payload: serde_json::Value = serde_json::from_str(&payload_text)
            .map_err(|e| SumsubError::MalformedPayload(e.to_string()))?;
        let token = payload
            .get("token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SumsubError::MalformedPayload("access token missing".into()))?
            .to_string();
        let expires_at = (Utc::now() + chrono::Duration::seconds(TTL_SECS)).naive_utc();

        Ok(AccessToken {
            token,
            user_id: qent_user_id.to_string(),
            level_name: level_name.to_string(),
            expires_at,
        })
    }

    /// Verify an inbound webhook signature against the raw body, per
    /// §4.3 (signature-first discipline). Sumsub sends the hex digest
    /// in the `X-Payload-Digest` header, HMAC-SHA256 with the
    /// webhook secret.
    pub fn verify_webhook(
        &self,
        signature_header: &str,
        raw_body: &[u8],
    ) -> Result<(), SumsubError> {
        if self.webhook_secret.is_empty() {
            return Err(SumsubError::NotConfigured);
        }

        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(self.webhook_secret.as_bytes())
            .map_err(|_| SumsubError::InvalidSignature)?;
        mac.update(raw_body);
        let expected = mac.finalize().into_bytes();

        let provided = hex::decode(signature_header.trim()).map_err(|_| SumsubError::InvalidSignature)?;

        // Constant-time compare via slice equality on fixed-size
        // hashes (subtle would be stricter; SHA-256 length parity
        // here keeps the check timing-stable across the inner cmp).
        if provided.len() != expected.len() {
            return Err(SumsubError::InvalidSignature);
        }
        let mut diff = 0u8;
        for (a, b) in provided.iter().zip(expected.iter()) {
            diff |= a ^ b;
        }
        if diff != 0 {
            return Err(SumsubError::InvalidSignature);
        }
        Ok(())
    }

    /// Map a Sumsub webhook payload to a `SumsubDecision`. Only
    /// `applicantReviewed` events carry a verdict; everything else
    /// (pending, on-hold, status changes) maps to `Pending`.
    pub fn parse_decision(&self, raw_body: &[u8]) -> Result<SumsubDecision, SumsubError> {
        let payload: serde_json::Value = serde_json::from_slice(raw_body)
            .map_err(|e| SumsubError::MalformedPayload(e.to_string()))?;

        let applicant_id = payload
            .get("applicantId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SumsubError::MalformedPayload("missing applicantId".into()))?
            .to_string();
        let external_user_id = payload
            .get("externalUserId")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let event_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("");

        if event_type != "applicantReviewed" {
            return Ok(SumsubDecision::Pending {
                applicant_id,
                external_user_id,
            });
        }

        let review = payload
            .get("reviewResult")
            .ok_or_else(|| SumsubError::MalformedPayload("missing reviewResult".into()))?;
        let answer = review
            .get("reviewAnswer")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let reject_labels = review
            .get("rejectLabels")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let review_reject_type = review
            .get("reviewRejectType")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match answer {
            "GREEN" => Ok(SumsubDecision::Approved {
                applicant_id,
                external_user_id,
            }),
            "RED" => {
                let reason = if reject_labels.is_empty() {
                    "rejected".to_string()
                } else {
                    reject_labels
                        .iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(",")
                };
                let retryable = review_reject_type.eq_ignore_ascii_case("RETRY");
                Ok(SumsubDecision::Rejected {
                    applicant_id,
                    external_user_id,
                    reason,
                    retryable,
                })
            }
            _ => Ok(SumsubDecision::Pending {
                applicant_id,
                external_user_id,
            }),
        }
    }
}
