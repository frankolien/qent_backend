//! Alchemy webhook receiver — chain confirmations.
//!
//! V2 §4.1 step 15-17, §4.3, §13.7. Single inbound endpoint that
//! Alchemy hits when a USDC transfer to the escrow wallet
//! confirms. Discipline (per §3.5 and §4.3): verify HMAC
//! signature first, then look up the matching `payments` row by
//! `tx_hash`, then assert from/to/amount match, then transition
//! the booking. Anything failing returns a clean 4xx so Alchemy
//! retries.
//!
//! Idempotency is structural: the §9.2 `UNIQUE(tx_hash)` partial
//! index on `payments` makes a duplicate webhook + reconciler
//! race a no-op on the second arrival.
//!
//! Stub for Week 0 — route exists, returns 200, no DB writes yet.

use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::PgPool;

pub async fn usdc_receive(
    _req: HttpRequest,
    _pool: web::Data<PgPool>,
    _body: web::Bytes,
) -> HttpResponse {
    // §4.1 step 16:
    //   1. Verify Alchemy HMAC signature over raw body.
    //   2. Parse the activity payload — chain, tx_hash, from, to,
    //      token contract, value.
    //   3. Reject if token contract != canonical Base USDC (§13.2).
    //   4. Look up `payments` by tx_hash (or by destination + amount
    //      if tx_hash not yet submitted; submit-tx race window).
    //   5. Assert from == renter.wallet, to == ESCROW, value ==
    //      payment.amount_usdc. On any mismatch → status=mismatch,
    //      admin queue.
    //   6. UPDATE payments SET status='success', tx_hash=...
    //      INSERT idempotent — on UNIQUE violation, no-op.
    //   7. Call transition_booking_to_paid(booking_id).
    //   8. Emit WebSocket event + push notification (fire-and-forget).
    HttpResponse::Ok().finish()
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/webhooks/alchemy/usdc-receive",
        web::post().to(usdc_receive),
    );
}
