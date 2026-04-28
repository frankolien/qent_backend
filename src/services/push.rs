use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

const TOKEN_REFRESH_SAFETY_SECS: i64 = 300;
const FCM_SCOPE: &str = "https://www.googleapis.com/auth/firebase.messaging";
const GOOGLE_OAUTH_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

#[derive(Debug, Deserialize)]
struct ServiceAccount {
    project_id: String,
    private_key: String,
    client_email: String,
}

#[derive(Debug, Serialize)]
struct OAuthClaims {
    iss: String,
    scope: String,
    aud: String,
    iat: i64,
    exp: i64,
}

#[derive(Debug, Deserialize)]
struct OAuthResponse {
    access_token: String,
    expires_in: i64,
}

#[derive(Debug, Clone)]
struct CachedToken {
    access_token: String,
    expires_at: chrono::DateTime<Utc>,
}

#[derive(Clone)]
pub struct PushService {
    project_id: String,
    private_key: EncodingKey,
    client_email: String,
    http: reqwest::Client,
    cached_token: Arc<RwLock<Option<CachedToken>>>,
}

impl PushService {
    pub fn from_env() -> Result<Self, String> {
        // Prefer inline JSON env var (Render / serverless friendly).
        // Fall back to a file path for local dev.
        let json = if let Ok(inline) = std::env::var("FIREBASE_SERVICE_ACCOUNT_JSON") {
            inline
        } else {
            let path = std::env::var("FIREBASE_SERVICE_ACCOUNT_PATH").map_err(|_| {
                "Neither FIREBASE_SERVICE_ACCOUNT_JSON nor FIREBASE_SERVICE_ACCOUNT_PATH is set"
                    .to_string()
            })?;
            std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read {}: {}", path, e))?
        };

        let sa: ServiceAccount = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse service account JSON: {}", e))?;

        let private_key = EncodingKey::from_rsa_pem(sa.private_key.as_bytes())
            .map_err(|e| format!("Failed to parse private key: {}", e))?;

        Ok(Self {
            project_id: sa.project_id,
            private_key,
            client_email: sa.client_email,
            http: reqwest::Client::new(),
            cached_token: Arc::new(RwLock::new(None)),
        })
    }

    async fn get_access_token(&self) -> Result<String, String> {
        if let Some(cached) = self.cached_token.read().await.as_ref() {
            if cached.expires_at > Utc::now() + Duration::seconds(TOKEN_REFRESH_SAFETY_SECS) {
                return Ok(cached.access_token.clone());
            }
        }

        let now = Utc::now();
        let claims = OAuthClaims {
            iss: self.client_email.clone(),
            scope: FCM_SCOPE.to_string(),
            aud: GOOGLE_OAUTH_TOKEN_URL.to_string(),
            iat: now.timestamp(),
            exp: (now + Duration::hours(1)).timestamp(),
        };

        let jwt = encode(&Header::new(Algorithm::RS256), &claims, &self.private_key)
            .map_err(|e| format!("JWT signing failed: {}", e))?;

        let response = self
            .http
            .post(GOOGLE_OAUTH_TOKEN_URL)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await
            .map_err(|e| format!("OAuth request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("OAuth token request failed: {} {}", status, body));
        }

        let token_response: OAuthResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse OAuth response: {}", e))?;

        let cached = CachedToken {
            access_token: token_response.access_token.clone(),
            expires_at: Utc::now() + Duration::seconds(token_response.expires_in),
        };
        *self.cached_token.write().await = Some(cached);

        Ok(token_response.access_token)
    }

    pub async fn send_to_user(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        title: &str,
        body: &str,
        data: serde_json::Value,
    ) {
        let tokens: Vec<(String, String)> = match sqlx::query_as::<_, (String, String)>(
            "SELECT token, platform FROM device_tokens WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        {
            Ok(rows) => rows,
            Err(e) => {
                log::error!("Failed to fetch device tokens for user {}: {}", user_id, e);
                return;
            }
        };

        if tokens.is_empty() {
            log::debug!("No device tokens for user {}, skipping push", user_id);
            return;
        }

        let access_token = match self.get_access_token().await {
            Ok(t) => t,
            Err(e) => {
                log::error!("Failed to get FCM access token: {}", e);
                return;
            }
        };

        let url = format!(
            "https://fcm.googleapis.com/v1/projects/{}/messages:send",
            self.project_id
        );

        for (token, _platform) in tokens {
            let payload = serde_json::json!({
                "message": {
                    "token": token,
                    "notification": { "title": title, "body": body },
                    "data": data_to_string_map(&data),
                }
            });

            let result = self
                .http
                .post(&url)
                .bearer_auth(&access_token)
                .json(&payload)
                .send()
                .await;

            match result {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        log::info!("Push delivered to user {}", user_id);
                    } else if status == reqwest::StatusCode::NOT_FOUND
                        || status == reqwest::StatusCode::GONE
                    {
                        let _ = sqlx::query("DELETE FROM device_tokens WHERE token = $1")
                            .bind(&token)
                            .execute(pool)
                            .await;
                        log::info!("Pruned dead FCM token for user {}", user_id);
                    } else {
                        let body = resp.text().await.unwrap_or_default();
                        log::warn!("FCM send failed ({}): {}", status, body);
                    }
                }
                Err(e) => log::error!("FCM HTTP error: {}", e),
            }
        }
    }
}

fn data_to_string_map(data: &serde_json::Value) -> serde_json::Value {
    match data {
        serde_json::Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                let s = match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                out.insert(k.clone(), serde_json::Value::String(s));
            }
            serde_json::Value::Object(out)
        }
        _ => serde_json::json!({}),
    }
}
