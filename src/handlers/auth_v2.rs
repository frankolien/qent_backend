//! V2 auth surface.
//!
//! Currently one route: `POST /api/auth/privy` — exchange a Privy
//! access token for a Qent JWT, finding-or-creating the `users` row
//! and (best-effort) the `wallets` row on first sign-in.
//!
//! Per V2 §6.4-§6.5: the mobile client owns the OAuth/email flow via
//! Privy's SDK and sends us only the signed session token. We verify
//! the signature against Privy's JWKS, pull linked-account details
//! the first time we see a `privy_user_id`, and provision the
//! embedded wallet behind the scenes.

use actix_web::{web, HttpResponse};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::{AuthResponseWithRefresh, Claims, User, UserPublic, UserRole, VerificationStatus};
use crate::services::wallet::{WalletClient, WalletError};
use crate::services::AppConfig;

#[derive(Debug, Deserialize, ToSchema)]
pub struct PrivyAuthRequest {
    pub privy_token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PrivyAuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub user: UserPublic,
    pub kyc_tier: i32,
    pub country: Option<String>,
    pub wallet_address: Option<String>,
}

fn generate_refresh_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..64)
        .map(|_| {
            let idx = rng.gen_range(0..36);
            if idx < 10 {
                (b'0' + idx) as char
            } else {
                (b'a' + idx - 10) as char
            }
        })
        .collect()
}

fn sign_qent_jwt(user_id: Uuid, role: UserRole, secret: &str) -> Result<String, HttpResponse> {
    let claims = Claims {
        sub: user_id,
        role,
        exp: (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| {
        log::error!("Qent JWT encode failed: {e}");
        HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Failed to issue session token"}))
    })
}

#[utoipa::path(
    post,
    path = "/api/auth/privy",
    tag = "Auth",
    request_body = PrivyAuthRequest,
    responses(
        (status = 200, description = "Privy session exchanged for Qent JWT", body = PrivyAuthResponse),
        (status = 401, description = "Privy session invalid"),
        (status = 503, description = "Privy not configured on the server"),
    ),
)]
pub async fn exchange_privy(
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    wallet: web::Data<WalletClient>,
    body: web::Json<PrivyAuthRequest>,
) -> HttpResponse {
    let session = match wallet.verify_session(&body.privy_token).await {
        Ok(s) => s,
        Err(WalletError::NotConfigured) => {
            return HttpResponse::ServiceUnavailable()
                .json(serde_json::json!({"error": "Privy not configured"}));
        }
        Err(WalletError::InvalidSession(reason)) => {
            log::warn!("Privy session verification failed: {reason}");
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Invalid Privy session"}));
        }
        Err(e) => {
            log::error!("Privy verify_session error: {e}");
            return HttpResponse::BadGateway()
                .json(serde_json::json!({"error": "Privy unreachable"}));
        }
    };

    let existing = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE privy_user_id = $1 AND is_active = true",
    )
    .bind(&session.privy_user_id)
    .fetch_optional(pool.get_ref())
    .await;

    let (user, wallet_address) = match existing {
        Ok(Some(u)) => {
            let addr = sqlx::query_scalar::<_, String>(
                "SELECT address FROM wallets WHERE user_id = $1",
            )
            .bind(u.id)
            .fetch_optional(pool.get_ref())
            .await
            .ok()
            .flatten();
            (u, addr)
        }
        Ok(None) => {
            let details = match wallet.fetch_user_details(&session.privy_user_id).await {
                Ok(d) => d,
                Err(e) => {
                    log::error!("Privy fetch_user_details failed: {e}");
                    return HttpResponse::BadGateway()
                        .json(serde_json::json!({"error": "Could not load Privy user"}));
                }
            };

            // Privy may return a user without an email if they signed
            // up with a wallet only. We require an email to keep V1
            // notifications working; fall back to a synthetic address
            // so the unique constraint holds.
            let email = details.email.clone().unwrap_or_else(|| {
                format!("{}@privy.qent.local", session.privy_user_id)
            });
            // Privy DIDs don't carry a display name. Default to "" so
            // the onboarding step that captures full_name can populate
            // it later without a constraint dance.
            let full_name = String::new();

            let id = Uuid::new_v4();
            let now = Utc::now().naive_utc();

            let insert = sqlx::query(
                r#"INSERT INTO users (
                    id, email, password_hash, full_name, role,
                    verification_status, wallet_balance, is_active,
                    country, privy_user_id, auth_provider, kyc_tier,
                    created_at, updated_at
                ) VALUES (
                    $1, $2, NULL, $3, $4, $5, 0.0, true,
                    NULL, $6, $7, 0,
                    $8, $8
                )"#,
            )
            .bind(id)
            .bind(&email)
            .bind(&full_name)
            .bind(UserRole::Renter)
            .bind(VerificationStatus::Pending)
            .bind(&session.privy_user_id)
            .bind(&details.auth_provider)
            .bind(now)
            .execute(pool.get_ref())
            .await;

            if let Err(e) = insert {
                let msg = e.to_string();
                if msg.contains("users_email_key") || msg.contains("duplicate key") {
                    return HttpResponse::Conflict().json(serde_json::json!({
                        "error": "Email already registered with another sign-in method"
                    }));
                }
                log::error!("Failed to create Privy user: {e}");
                return HttpResponse::InternalServerError()
                    .json(serde_json::json!({"error": "Failed to create account"}));
            }

            // Best-effort wallet provisioning. A Privy outage shouldn't
            // block sign-in — the user can still browse + KYC; the
            // wallet is required only at booking time, and we retry
            // lazily there. The `wallets` row is what the rest of the
            // app reads from.
            let mut wallet_address: Option<String> = None;
            match wallet.create_wallet(&session.privy_user_id).await {
                Ok(w) => {
                    let _ = sqlx::query(
                        r#"INSERT INTO wallets (user_id, address, chain, privy_wallet_id)
                           VALUES ($1, $2, 'base', $3)
                           ON CONFLICT (user_id) DO NOTHING"#,
                    )
                    .bind(id)
                    .bind(&w.address)
                    .bind(&w.privy_wallet_id)
                    .execute(pool.get_ref())
                    .await;
                    wallet_address = Some(w.address);
                }
                Err(WalletError::NotConfigured) => {
                    log::warn!("Privy create_wallet skipped: not configured");
                }
                Err(e) => {
                    log::warn!("Privy create_wallet failed (will retry on booking): {e}");
                }
            }

            let user = match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
                .bind(id)
                .fetch_one(pool.get_ref())
                .await
            {
                Ok(u) => u,
                Err(e) => {
                    log::error!("Failed to reload created Privy user: {e}");
                    return HttpResponse::InternalServerError()
                        .json(serde_json::json!({"error": "Failed to create account"}));
                }
            };
            (user, wallet_address)
        }
        Err(e) => {
            log::error!("DB error looking up Privy user: {e}");
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Internal server error"}));
        }
    };

    let token = match sign_qent_jwt(user.id, user.role.clone(), &config.jwt_secret) {
        Ok(t) => t,
        Err(resp) => return resp,
    };
    let refresh = generate_refresh_token();
    let _ = sqlx::query("UPDATE users SET refresh_token = $1 WHERE id = $2")
        .bind(&refresh)
        .bind(user.id)
        .execute(pool.get_ref())
        .await;

    let kyc_tier = sqlx::query_scalar::<_, i32>("SELECT kyc_tier FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_optional(pool.get_ref())
        .await
        .ok()
        .flatten()
        .unwrap_or(0);

    let country_opt = user.country.clone();

    HttpResponse::Ok().json(AuthV2Body {
        wrapped: AuthResponseWithRefresh {
            token,
            refresh_token: refresh,
            user: user.into(),
        },
        kyc_tier,
        country: country_opt,
        wallet_address,
    })
}

#[derive(Serialize)]
struct AuthV2Body {
    #[serde(flatten)]
    wrapped: AuthResponseWithRefresh,
    kyc_tier: i32,
    country: Option<String>,
    wallet_address: Option<String>,
}
