//! Sumsub webhook receiver — KYC decisions.
//!
//! V2 §4.3, §5.5, §6.7. Sumsub posts decision events here when an
//! applicant's review completes. Same discipline as the Alchemy
//! webhook: verify HMAC signature first, then update the user's
//! `kyc_tier` based on the decision.
//!
//! Maps Sumsub decision → tier transition:
//!   Approved      → kyc_tier 0 → 1 (for the standard booking flow)
//!                   kyc_tier 1 → 2 only via admin listing approval;
//!                                 Sumsub alone does not promote to
//!                                 tier_2.
//!   Rejected      → no tier change; record reason for the admin
//!                   queue + user-visible "try again" path.
//!   Pending       → no-op; we are still waiting.
//!
//! Stub for Week 0 — route exists, returns 200, no DB writes yet.

use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::PgPool;

pub async fn decision(
    _req: HttpRequest,
    _pool: web::Data<PgPool>,
    _body: web::Bytes,
) -> HttpResponse {
    // §6.7 trigger handling:
    //   1. Verify Sumsub HMAC signature over raw body.
    //   2. Parse the decision payload via services::sumsub.
    //   3. Look up users.sumsub_applicant_id.
    //   4. On Approved: UPDATE users SET kyc_tier = 1 WHERE
    //      sumsub_applicant_id = $1 AND kyc_tier < 1.
    //      (Monotonic: never downgrade through this path.)
    //   5. On Rejected: leave kyc_tier alone, record reason for
    //      the user-visible "try again" flow + admin queue.
    //   6. On Pending: no-op.
    //   7. Emit WebSocket event so the polling mobile client
    //      pulls the updated tier.
    HttpResponse::Ok().finish()
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/webhooks/sumsub", web::post().to(decision));
}
