//! Privy embedded-wallet operations.
//!
//! Per V2 §3.3 and §6.5, every user gets an app-controlled wallet on
//! Base provisioned through Privy during Phase 4 of onboarding. This
//! module is the server-side wrapper around the Privy REST API:
//! verify a session token, look up the user's linked accounts, create
//! a wallet, and (Week 2) instruct escrow transfers.
//!
//! The `PRIVY_APP_SECRET` never leaves this module; handler code
//! calls into typed methods. All Privy API calls go through here.

use std::sync::RwLock;
use std::time::{Duration, Instant};

use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const PRIVY_ISSUER: &str = "privy.io";
const JWKS_CACHE_TTL: Duration = Duration::from_secs(60 * 60);
const PRIVY_API_BASE: &str = "https://auth.privy.io/api/v1";

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
    #[error("Privy session token failed JWKS verification: {0}")]
    InvalidSession(String),
}

/// One Privy-managed embedded wallet on Base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivyWallet {
    pub privy_wallet_id: String,
    pub address: String,
    pub chain: String,
}

/// The minimum the JWT alone tells us: the Privy user id (`sub`).
/// Linked email / provider come from `fetch_user_details` because
/// Privy does not embed them in the access token.
#[derive(Debug, Clone)]
pub struct VerifiedPrivySession {
    pub privy_user_id: String,
}

/// Linked-account snapshot returned by Privy's `/users/{id}` endpoint.
/// We collapse the many account types Privy supports into one of
/// `google | apple | email` plus the address — anything else falls
/// back to `email` since we always have a Privy user id either way.
#[derive(Debug, Clone)]
pub struct PrivyUserDetails {
    pub privy_user_id: String,
    pub email: Option<String>,
    pub auth_provider: String,
}

#[derive(Clone)]
pub struct WalletClient {
    http: Client,
    app_id: String,
    app_secret: String,
    jwks_url: String,
}

struct CachedJwks {
    keys: Vec<PrivyJwk>,
    fetched_at: Instant,
}

static JWKS_CACHE: RwLock<Option<CachedJwks>> = RwLock::new(None);

#[derive(Debug, Clone, Deserialize)]
struct PrivyJwk {
    kty: String,
    kid: String,
    crv: Option<String>,
    x: Option<String>,
    y: Option<String>,
    alg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PrivyJwks {
    keys: Vec<PrivyJwk>,
}

#[derive(Debug, Deserialize)]
struct PrivyAccessClaims {
    sub: String,
}

impl WalletClient {
    pub fn new(app_id: String, app_secret: String, jwks_url: String) -> Self {
        // The default JWKS URL has `_` as a placeholder for the
        // app id (set in `AppConfig::from_env`). Substitute now so
        // `verify_session` doesn't have to.
        let jwks_url = if jwks_url.contains("/_/") && !app_id.is_empty() {
            jwks_url.replace("/_/", &format!("/{app_id}/"))
        } else {
            jwks_url
        };
        Self {
            http: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("reqwest client"),
            app_id,
            app_secret,
            jwks_url,
        }
    }

    fn ensure_configured(&self) -> Result<(), WalletError> {
        if self.app_id.is_empty() || self.app_secret.is_empty() {
            return Err(WalletError::NotConfigured);
        }
        Ok(())
    }

    async fn fetch_jwks(&self) -> Result<Vec<PrivyJwk>, WalletError> {
        log::info!("Fetching Privy JWKS from {}", self.jwks_url);
        let resp = self.http.get(&self.jwks_url).send().await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(WalletError::BadResponse { status, body });
        }
        let jwks: PrivyJwks = resp.json().await?;
        Ok(jwks.keys)
    }

    async fn get_jwks(&self, force_refresh: bool) -> Result<Vec<PrivyJwk>, WalletError> {
        if !force_refresh {
            if let Ok(guard) = JWKS_CACHE.read() {
                if let Some(c) = guard.as_ref() {
                    if c.fetched_at.elapsed() < JWKS_CACHE_TTL {
                        return Ok(c.keys.clone());
                    }
                }
            }
        }
        let keys = self.fetch_jwks().await?;
        if let Ok(mut guard) = JWKS_CACHE.write() {
            *guard = Some(CachedJwks {
                keys: keys.clone(),
                fetched_at: Instant::now(),
            });
        }
        Ok(keys)
    }

    /// Validate a Privy session token from the mobile client and
    /// return the verified Privy user id. The token is an ES256 JWT
    /// signed with one of the keys at `privy_jwks_url`; `iss` is
    /// `privy.io` and `aud` is our Privy app id.
    pub async fn verify_session(
        &self,
        privy_token: &str,
    ) -> Result<VerifiedPrivySession, WalletError> {
        self.ensure_configured()?;

        let header = decode_header(privy_token)
            .map_err(|e| WalletError::InvalidSession(format!("bad header: {e}")))?;
        let kid = header
            .kid
            .ok_or_else(|| WalletError::InvalidSession("missing kid".into()))?;

        let mut keys = self.get_jwks(false).await?;
        let jwk = match find_key(&keys, &kid) {
            Some(k) => k.clone(),
            None => {
                keys = self.get_jwks(true).await?;
                find_key(&keys, &kid)
                    .ok_or_else(|| {
                        WalletError::InvalidSession(format!("no JWK matching kid {kid}"))
                    })?
                    .clone()
            }
        };

        let x = jwk
            .x
            .as_deref()
            .ok_or_else(|| WalletError::InvalidSession("JWK missing x".into()))?;
        let y = jwk
            .y
            .as_deref()
            .ok_or_else(|| WalletError::InvalidSession("JWK missing y".into()))?;

        let decoding_key = DecodingKey::from_ec_components(x, y)
            .map_err(|e| WalletError::InvalidSession(format!("decoding key: {e}")))?;

        let mut validation = Validation::new(Algorithm::ES256);
        validation.set_issuer(&[PRIVY_ISSUER]);
        validation.set_audience(&[self.app_id.as_str()]);

        let data = decode::<PrivyAccessClaims>(privy_token, &decoding_key, &validation)
            .map_err(|e| WalletError::InvalidSession(format!("verify: {e}")))?;

        Ok(VerifiedPrivySession {
            privy_user_id: data.claims.sub,
        })
    }

    /// Fetch the user's linked accounts from Privy's user API.
    /// Called on first sign-in so we can persist email + provider on
    /// the Qent `users` row. Subsequent sign-ins skip this.
    pub async fn fetch_user_details(
        &self,
        privy_user_id: &str,
    ) -> Result<PrivyUserDetails, WalletError> {
        self.ensure_configured()?;

        let url = format!("{PRIVY_API_BASE}/users/{privy_user_id}");
        let resp = self
            .http
            .get(&url)
            .basic_auth(&self.app_id, Some(&self.app_secret))
            .header("privy-app-id", &self.app_id)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(WalletError::BadResponse { status, body });
        }

        let payload: serde_json::Value = resp.json().await?;
        let accounts = payload
            .get("linked_accounts")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        // First-wins for provider; email is the first verified address
        // we find. Privy account "type" values seen in the wild:
        // google_oauth, apple_oauth, email, phone, wallet.
        let mut auth_provider = "email".to_string();
        let mut email: Option<String> = None;

        for acct in &accounts {
            let ty = acct.get("type").and_then(|v| v.as_str()).unwrap_or("");
            match ty {
                "google_oauth" if auth_provider == "email" => {
                    auth_provider = "google".into();
                    if email.is_none() {
                        email = acct
                            .get("email")
                            .and_then(|v| v.as_str())
                            .map(str::to_owned);
                    }
                }
                "apple_oauth" if auth_provider == "email" => {
                    auth_provider = "apple".into();
                    if email.is_none() {
                        email = acct
                            .get("email")
                            .and_then(|v| v.as_str())
                            .map(str::to_owned);
                    }
                }
                "email" if email.is_none() => {
                    email = acct
                        .get("address")
                        .and_then(|v| v.as_str())
                        .map(str::to_owned);
                }
                _ => {}
            }
        }

        Ok(PrivyUserDetails {
            privy_user_id: privy_user_id.to_string(),
            email,
            auth_provider,
        })
    }

    /// Create an app-controlled embedded wallet for a Privy user on
    /// Base. Called once per user during Phase 4 of onboarding.
    pub async fn create_wallet(&self, privy_user_id: &str) -> Result<PrivyWallet, WalletError> {
        self.ensure_configured()?;

        let url = format!("{PRIVY_API_BASE}/wallets");
        let resp = self
            .http
            .post(&url)
            .basic_auth(&self.app_id, Some(&self.app_secret))
            .header("privy-app-id", &self.app_id)
            .json(&serde_json::json!({
                "owner": { "user_id": privy_user_id },
                "chain_type": "ethereum",
            }))
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(WalletError::BadResponse { status, body });
        }

        let payload: serde_json::Value = resp.json().await?;
        let id = payload
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WalletError::BadResponse {
                status,
                body: "wallet response missing id".into(),
            })?
            .to_string();
        let address = payload
            .get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WalletError::BadResponse {
                status,
                body: "wallet response missing address".into(),
            })?
            .to_string();

        Ok(PrivyWallet {
            privy_wallet_id: id,
            address,
            chain: "base".to_string(),
        })
    }

    /// Look up a wallet by its Privy id. The local `wallets` table is
    /// authoritative for hot reads; this is for repair / out-of-band.
    pub async fn fetch_wallet(&self, privy_wallet_id: &str) -> Result<PrivyWallet, WalletError> {
        self.ensure_configured()?;

        let url = format!("{PRIVY_API_BASE}/wallets/{privy_wallet_id}");
        let resp = self
            .http
            .get(&url)
            .basic_auth(&self.app_id, Some(&self.app_secret))
            .header("privy-app-id", &self.app_id)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(WalletError::BadResponse { status, body });
        }

        let payload: serde_json::Value = resp.json().await?;
        let id = payload
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or(privy_wallet_id)
            .to_string();
        let address = payload
            .get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WalletError::BadResponse {
                status,
                body: "wallet response missing address".into(),
            })?
            .to_string();

        Ok(PrivyWallet {
            privy_wallet_id: id,
            address,
            chain: "base".to_string(),
        })
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

fn find_key<'a>(keys: &'a [PrivyJwk], kid: &str) -> Option<&'a PrivyJwk> {
    keys.iter().find(|k| {
        k.kid == kid
            && k.kty == "EC"
            && k.crv.as_deref().unwrap_or("P-256") == "P-256"
            && k.alg.as_deref().unwrap_or("ES256") == "ES256"
    })
}

