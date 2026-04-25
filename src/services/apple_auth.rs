use std::sync::RwLock;
use std::time::{Duration, Instant};

use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;

const APPLE_JWKS_URL: &str = "https://appleid.apple.com/auth/keys";
const APPLE_ISSUER: &str = "https://appleid.apple.com";
const JWKS_CACHE_TTL: Duration = Duration::from_secs(60 * 60); // 1 hour

#[derive(Debug, Clone, Deserialize)]
struct AppleJwk {
    kty: String,
    kid: String,
    #[serde(rename = "use")]
    _use: Option<String>,
    alg: Option<String>,
    n: String,
    e: String,
}

#[derive(Debug, Deserialize)]
struct AppleJwks {
    keys: Vec<AppleJwk>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppleIdClaims {
    /// Stable unique user ID (per team). Use this as the primary Apple identifier.
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
    pub email: Option<String>,
    pub email_verified: Option<serde_json::Value>,
    pub is_private_email: Option<serde_json::Value>,
}

struct CachedJwks {
    keys: Vec<AppleJwk>,
    fetched_at: Instant,
}

static JWKS_CACHE: RwLock<Option<CachedJwks>> = RwLock::new(None);

async fn fetch_jwks() -> Result<Vec<AppleJwk>, String> {
    log::info!("Fetching Apple JWKS from {APPLE_JWKS_URL}");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let resp = client
        .get(APPLE_JWKS_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch Apple JWKS: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Apple JWKS returned status {}", resp.status()));
    }

    let jwks: AppleJwks = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse Apple JWKS: {e}"))?;

    log::info!("Apple JWKS fetched: {} keys", jwks.keys.len());
    Ok(jwks.keys)
}

async fn get_jwks(force_refresh: bool) -> Result<Vec<AppleJwk>, String> {
    if !force_refresh {
        if let Ok(guard) = JWKS_CACHE.read() {
            if let Some(c) = guard.as_ref() {
                if c.fetched_at.elapsed() < JWKS_CACHE_TTL {
                    return Ok(c.keys.clone());
                }
            }
        }
    }

    let keys = fetch_jwks().await?;
    if let Ok(mut guard) = JWKS_CACHE.write() {
        *guard = Some(CachedJwks {
            keys: keys.clone(),
            fetched_at: Instant::now(),
        });
    }
    Ok(keys)
}

fn find_key<'a>(keys: &'a [AppleJwk], kid: &str) -> Option<&'a AppleJwk> {
    keys.iter().find(|k| k.kid == kid && k.kty == "RSA")
}

/// Verify an Apple `identityToken` and return the claims.
/// Validates signature (via Apple's JWKS), issuer, audience (must equal `bundle_id`), and expiry.
pub async fn verify_identity_token(
    identity_token: &str,
    bundle_id: &str,
) -> Result<AppleIdClaims, String> {
    if bundle_id.is_empty() {
        return Err("APPLE_BUNDLE_ID is not configured".to_string());
    }

    let header = decode_header(identity_token)
        .map_err(|e| format!("Invalid token header: {e}"))?;

    let kid = header.kid.ok_or_else(|| "Token missing kid".to_string())?;

    // Try cached keys first; if the kid isn't present, refresh once (Apple rotates keys).
    let mut keys = get_jwks(false).await?;
    let jwk = match find_key(&keys, &kid) {
        Some(k) => k,
        None => {
            keys = get_jwks(true).await?;
            find_key(&keys, &kid)
                .ok_or_else(|| format!("No Apple JWK matching kid {kid}"))?
        }
    };

    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
        .map_err(|e| format!("Failed to build decoding key: {e}"))?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[APPLE_ISSUER]);
    validation.set_audience(&[bundle_id]);

    let data = decode::<AppleIdClaims>(identity_token, &decoding_key, &validation)
        .map_err(|e| format!("Token verification failed: {e}"))?;

    Ok(data.claims)
}
