//! Sumsub webhook receiver — KYC decisions.
//!
//! V2 §4.3, §5.5, §6.7. Signature-first discipline: verify the
//! `X-Payload-Digest` HMAC over the raw body before doing any DB
//! work, then look up the user by `sumsub_applicant_id` and flip
//! `kyc_tier`.
//!
//! Tier transitions:
//!   Approved + tier == 0  → tier = 1 (renter unlocks booking)
//!   Approved + tier >= 1  → no-op (monotonic ratchet, never
//!                            downgrade; tier_2 promotions are
//!                            admin-driven via listing approval).
//!   Rejected              → leave tier alone; record reason for
//!                            the user-visible retry path + admin
//!                            queue. (Persistence of the rejection
//!                            text lands with the admin queue UI;
//!                            for Week 1 the user re-attempts via
//!                            the WebSDK directly.)
//!   Pending               → no-op.

use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::PgPool;

use crate::services::sumsub::{SumsubClient, SumsubDecision, SumsubError};

const SIGNATURE_HEADER: &str = "x-payload-digest";

pub async fn decision(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    sumsub: web::Data<SumsubClient>,
    body: web::Bytes,
) -> HttpResponse {
    let sig = match req
        .headers()
        .get(SIGNATURE_HEADER)
        .and_then(|v| v.to_str().ok())
    {
        Some(s) => s.to_string(),
        None => {
            log::warn!("Sumsub webhook missing {SIGNATURE_HEADER}");
            return HttpResponse::Unauthorized().finish();
        }
    };

    if let Err(e) = sumsub.verify_webhook(&sig, &body) {
        match e {
            SumsubError::NotConfigured => {
                log::warn!("Sumsub webhook arrived but secret not configured");
                return HttpResponse::ServiceUnavailable().finish();
            }
            _ => {
                log::warn!("Sumsub webhook signature invalid");
                return HttpResponse::Unauthorized().finish();
            }
        }
    }

    let decision = match sumsub.parse_decision(&body) {
        Ok(d) => d,
        Err(e) => {
            log::warn!("Sumsub webhook parse failed: {e}");
            return HttpResponse::BadRequest().finish();
        }
    };

    match decision {
        SumsubDecision::Approved {
            applicant_id,
            external_user_id,
        } => {
            // Match by external_user_id first (it's our Qent UUID).
            // Fall back to applicant_id for the case where we have an
            // applicant row from a manual seed without externalUserId.
            let target = if let Ok(uid) = uuid::Uuid::parse_str(&external_user_id) {
                Some(uid)
            } else {
                None
            };

            let res = if let Some(uid) = target {
                sqlx::query(
                    r#"UPDATE users SET
                         kyc_tier = GREATEST(kyc_tier, 1),
                         sumsub_applicant_id = COALESCE(sumsub_applicant_id, $2),
                         updated_at = NOW()
                       WHERE id = $1"#,
                )
                .bind(uid)
                .bind(&applicant_id)
                .execute(pool.get_ref())
                .await
            } else {
                sqlx::query(
                    r#"UPDATE users SET
                         kyc_tier = GREATEST(kyc_tier, 1),
                         updated_at = NOW()
                       WHERE sumsub_applicant_id = $1"#,
                )
                .bind(&applicant_id)
                .execute(pool.get_ref())
                .await
            };

            match res {
                Ok(r) if r.rows_affected() == 0 => {
                    log::warn!("Sumsub approval for unknown applicant {applicant_id}");
                    // Still 200 so Sumsub doesn't retry — we have
                    // nothing to do, and a 4xx would imply we
                    // expected to find this user.
                    HttpResponse::Ok().finish()
                }
                Ok(_) => HttpResponse::Ok().finish(),
                Err(e) => {
                    log::error!("Sumsub tier flip failed: {e}");
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        SumsubDecision::Rejected {
            applicant_id,
            reason,
            retryable,
            ..
        } => {
            log::warn!(
                "Sumsub rejected applicant {applicant_id}: {reason} (retryable={retryable})"
            );
            HttpResponse::Ok().finish()
        }
        SumsubDecision::Pending { .. } => HttpResponse::Ok().finish(),
    }
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/webhooks/sumsub", web::post().to(decision));
}
