//! KYC handlers.
//!
//! V2 §6.7 — the mobile client calls `POST /api/kyc/access-token`
//! right before launching the Sumsub WebSDK on the first booking
//! attempt (or anywhere KYC is gated). The handler looks up the
//! user's country, resolves the Sumsub flow id from `countries`,
//! ensures a Sumsub applicant exists, persists its id to the user
//! row, and returns a short-lived access token + the flow name.

use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::ToSchema;

use crate::models::Claims;
use crate::services::sumsub::{SumsubClient, SumsubError};

#[derive(Debug, Serialize, ToSchema)]
pub struct AccessTokenResponse {
    pub token: String,
    pub level_name: String,
    pub expires_at: chrono::NaiveDateTime,
}

#[utoipa::path(
    post,
    path = "/api/kyc/access-token",
    tag = "KYC",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Short-lived Sumsub WebSDK token", body = AccessTokenResponse),
        (status = 401, description = "Unauthorized"),
        (status = 412, description = "Country not set on the user"),
        (status = 503, description = "Sumsub not configured"),
    ),
)]
pub async fn access_token(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    sumsub: web::Data<SumsubClient>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let row = sqlx::query_as::<_, (Option<String>, Option<String>)>(
        "SELECT country, sumsub_applicant_id FROM users WHERE id = $1",
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await;

    let (country, existing_applicant) = match row {
        Ok(Some(r)) => r,
        _ => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "User not found"}))
        }
    };

    let country = match country {
        Some(c) if !c.trim().is_empty() => c,
        _ => {
            return HttpResponse::PreconditionFailed()
                .json(serde_json::json!({"error": "Set your country first"}));
        }
    };

    let level_name = sqlx::query_scalar::<_, String>(
        "SELECT sumsub_level FROM countries WHERE iso2 = $1",
    )
    .bind(&country)
    .fetch_optional(pool.get_ref())
    .await
    .ok()
    .flatten()
    .unwrap_or_else(|| format!("qent-{}-v1", country.to_lowercase()));

    let user_id_str = claims.sub.to_string();

    // Reuse the persisted applicant id if we have one; otherwise
    // create on Sumsub and persist.
    let _applicant_id = match existing_applicant {
        Some(id) if !id.is_empty() => id,
        _ => match sumsub.ensure_applicant(&user_id_str, &level_name).await {
            Ok(id) => {
                let _ = sqlx::query(
                    "UPDATE users SET sumsub_applicant_id = $1, updated_at = NOW() WHERE id = $2",
                )
                .bind(&id)
                .bind(claims.sub)
                .execute(pool.get_ref())
                .await;
                id
            }
            Err(SumsubError::NotConfigured) => {
                return HttpResponse::ServiceUnavailable()
                    .json(serde_json::json!({"error": "Sumsub not configured"}));
            }
            Err(e) => {
                log::error!("Sumsub ensure_applicant failed: {e}");
                return HttpResponse::BadGateway()
                    .json(serde_json::json!({"error": "KYC provider unreachable"}));
            }
        },
    };

    match sumsub.create_access_token(&user_id_str, &level_name).await {
        Ok(t) => HttpResponse::Ok().json(AccessTokenResponse {
            token: t.token,
            level_name: t.level_name,
            expires_at: t.expires_at,
        }),
        Err(SumsubError::NotConfigured) => HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({"error": "Sumsub not configured"})),
        Err(e) => {
            log::error!("Sumsub create_access_token failed: {e}");
            HttpResponse::BadGateway()
                .json(serde_json::json!({"error": "KYC provider unreachable"}))
        }
    }
}
