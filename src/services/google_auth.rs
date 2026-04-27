use std::sync::RwLock;
use std::time::{Duration, Instant};

use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;

const GOOGLE_JWKS_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";
const GOOGLE_ISSUERS: [&str; 2] = ["accounts.google.com", "https://accounts.google.com"];
const JWKS_CACHE_TTL: Duration = Duration::from_secs(60 * 60); // 1 hour

#[derive(Debug, Clone, Deserialize)]
struct GoogleJwk {
    kty: String,
    kid: String,
    #[serde(rename = "use")]
    _use: Option<String>,
    alg: Option<String>,
    n: String,
    e: String,
}

#[derive(Debug, Deserialize)]
struct GoogleJwks {
    keys: Vec<GoogleJwk>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleIdClaims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub picture: Option<String>,
}

struct CachedJwks {
    keys: Vec<GoogleJwk>,
    fetched_at: Instant,
}

static JWKS_CACHE: RwLock<Option<CachedJwks>> = RwLock::new(None);

async fn fetch_jwks() -> Result<Vec<GoogleJwk>, String> {
    log::info!("Fetching Google JWKS from {GOOGLE_JWKS_URL}");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let resp = client
        .get(GOOGLE_JWKS_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch Google JWKS: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Google JWKS returned status {}", resp.status()));
    }

    let jwks: GoogleJwks = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse Google JWKS: {e}"))?;

    log::info!("Google JWKS fetched: {} keys", jwks.keys.len());
    Ok(jwks.keys)
}

async fn get_jwks(force_refresh: bool) -> Result<Vec<GoogleJwk>, String> {
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

fn find_key<'a>(keys: &'a [GoogleJwk], kid: &str) -> Option<&'a GoogleJwk> {
    keys.iter()
        .find(|k| k.kid == kid && k.kty == "RSA" && k.alg.as_deref().unwrap_or("RS256") == "RS256")
}

pub async fn verify_identity_token(
    identity_token: &str,
    client_ids: &[String],
) -> Result<GoogleIdClaims, String> {
    if client_ids.is_empty() {
        return Err("GOOGLE_CLIENT_IDS is not configured".to_string());
    }

    let header = decode_header(identity_token).map_err(|e| format!("Invalid token header: {e}"))?;

    let kid = header.kid.ok_or_else(|| "Token missing kid".to_string())?;

    let mut keys = get_jwks(false).await?;
    let jwk = match find_key(&keys, &kid) {
        Some(k) => k,
        None => {
            keys = get_jwks(true).await?;
            find_key(&keys, &kid).ok_or_else(|| format!("No Google JWK matching kid {kid}"))?
        }
    };

    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
        .map_err(|e| format!("Failed to build decoding key: {e}"))?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&GOOGLE_ISSUERS);
    validation.set_audience(client_ids);

    let data = decode::<GoogleIdClaims>(identity_token, &decoding_key, &validation)
        .map_err(|e| format!("Token verification failed: {e}"))?;

    Ok(data.claims)
}
