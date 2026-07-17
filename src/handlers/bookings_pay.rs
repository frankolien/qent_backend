//! V2 USDC payment surface — bolts the §4.1 USDC pipeline onto V1's
//! existing booking lifecycle.
//!
//! V1's flow stays intact:
//!   1. Renter `POST /api/bookings` → status `pending`
//!   2. Host `POST /api/bookings/{id}/action` `approve` → `approved`
//!   3. Renter taps Pay Now → **this module's** `POST /v2/bookings/{id}/pay`
//!      returns a payment intent; status flips to `pending_payment`
//!   4. Renter signs USDC transfer via Privy; `POST /v2/payments/{id}/submit-tx`
//!      binds tx_hash; webhook / reconciler flip booking to `paid`
//!
//! No V1 endpoint is replaced — this just adds the on-chain payment
//! handshake.

use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use chrono::{Duration as ChronoDuration, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

use crate::models::Claims;
use crate::services::AppConfig;

const SERVICE_FEE_PCT: &str = "0.12"; // §2.6 — Rev 2 take rate
const PAYMENT_INTENT_TTL_MIN: i64 = 15; // §4.1 step 6

#[derive(Debug, Serialize)]
pub struct PaymentIntent {
    pub booking_id: Uuid,
    pub payment_id: Uuid,
    pub destination: String,
    pub amount_usdc: String,
    pub chain: String,
    pub expires_at: chrono::DateTime<Utc>,
    pub host_price_usdc: String,
    pub service_fee_usdc: String,
    pub total_days: i32,
}

/// POST /api/v2/bookings/{id}/pay — renter requests a USDC payment
/// intent for an approved booking. Idempotent: if a `pending` payment
/// row already exists for this booking, returns it instead of creating
/// a duplicate (lets the mobile retry without orphaning a row).
pub async fn request_payment_intent(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Unauthorized"}))
        }
    };
    let booking_id = path.into_inner();

    if config.escrow_wallet_address.trim().is_empty() {
        return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({"error": "Escrow wallet not configured"}));
    }
    if config.escrow_halt {
        return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({"error": "Bookings paused"}));
    }

    // Pull booking + linked car USDC price.
    let row = match sqlx::query_as::<
        _,
        (Uuid, Uuid, Uuid, String, i32, Decimal),
    >(
        r#"SELECT b.id, b.renter_id, b.car_id, b.status, b.total_days,
                  COALESCE(c.price_per_day_usdc, 0)::numeric
           FROM bookings b
           JOIN cars c ON c.id = b.car_id
           WHERE b.id = $1"#,
    )
    .bind(booking_id)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": "Booking not found"}))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };
    let (_id, renter_id, _car_id, status, total_days, price_per_day_usdc) = row;

    if renter_id != claims.sub {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Not your booking"}));
    }
    // V1 statuses that mean "ready to pay" — `approved` is the
    // canonical post-host-approval state; `pending_payment` is the
    // V2-only retry state where a payment intent already exists.
    if !matches!(status.as_str(), "approved" | "pending_payment") {
        return HttpResponse::Conflict().json(serde_json::json!({
            "error": format!("Booking not ready for payment (status: {})", status)
        }));
    }
    if price_per_day_usdc <= Decimal::ZERO {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Car has no USDC price"}));
    }

    // Idempotency: if a still-pending payment exists for this booking,
    // return it instead of opening a second one.
    if let Ok(Some(existing)) = sqlx::query_as::<_, (Uuid, Decimal)>(
        "SELECT id, amount_usdc::numeric FROM payments WHERE booking_id = $1 AND status = 'pending' ORDER BY created_at DESC LIMIT 1",
    )
    .bind(booking_id)
    .fetch_optional(pool.get_ref())
    .await
    {
        let (payment_id, amount) = existing;
        let host_price = price_per_day_usdc * Decimal::from(total_days);
        let service_fee = (host_price * Decimal::from_str(SERVICE_FEE_PCT).unwrap()).round_dp(6);
        return HttpResponse::Ok().json(PaymentIntent {
            booking_id,
            payment_id,
            destination: config.escrow_wallet_address.clone(),
            amount_usdc: amount.to_string(),
            chain: "base".to_string(),
            expires_at: Utc::now() + ChronoDuration::minutes(PAYMENT_INTENT_TTL_MIN),
            host_price_usdc: host_price.to_string(),
            service_fee_usdc: service_fee.to_string(),
            total_days,
        });
    }

    // Compute math.
    let host_price = price_per_day_usdc * Decimal::from(total_days);
    let service_fee = (host_price * Decimal::from_str(SERVICE_FEE_PCT).unwrap()).round_dp(6);
    let total = (host_price + service_fee).round_dp(6);

    let payment_id = Uuid::new_v4();
    let expires_at = Utc::now() + ChronoDuration::minutes(PAYMENT_INTENT_TTL_MIN);

    // Tx: insert payment row + flip booking status (only if still
    // `approved` — pending_payment retries skip the flip).
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };

    if let Err(e) = sqlx::query(
        r#"INSERT INTO payments (
            id, booking_id, payer_id, amount, currency, provider, provider_reference,
            status, transaction_type, created_at, chain, amount_usdc, to_address
        ) VALUES (
            $1, $2, $3, $4::double precision, 'USDC', 'privy', NULL,
            'pending', 'payment'::transaction_type, NOW(), 'base', $5, $6
        )"#,
    )
    .bind(payment_id)
    .bind(booking_id)
    .bind(claims.sub)
    .bind(total.to_string().parse::<f64>().unwrap_or(0.0))
    .bind(total)
    .bind(&config.escrow_wallet_address)
    .execute(&mut *tx)
    .await
    {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()}));
    }

    // V1 status enum tolerates `pending_payment` per migration 027.
    if let Err(e) = sqlx::query(
        "UPDATE bookings SET status = 'pending_payment', total_usdc = $1, host_price_usdc = $2, service_fee_usdc = $3, updated_at = NOW() WHERE id = $4 AND status = 'approved'",
    )
    .bind(total)
    .bind(host_price)
    .bind(service_fee)
    .bind(booking_id)
    .execute(&mut *tx)
    .await
    {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()}));
    }

    if let Err(e) = tx.commit().await {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()}));
    }

    HttpResponse::Ok().json(PaymentIntent {
        booking_id,
        payment_id,
        destination: config.escrow_wallet_address.clone(),
        amount_usdc: total.to_string(),
        chain: "base".to_string(),
        expires_at,
        host_price_usdc: host_price.to_string(),
        service_fee_usdc: service_fee.to_string(),
        total_days,
    })
}

#[derive(Debug, serde::Deserialize)]
pub struct SubmitTxRequest {
    pub tx_hash: String,
    pub from_address: String,
}

/// POST /api/v2/payments/{id}/submit-tx — §4.1 step 13. Renter
/// binds the on-chain tx hash to the payment row so the Alchemy
/// webhook + reconciler can correlate confirmations.
pub async fn submit_tx(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<SubmitTxRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let payment_id = path.into_inner();
    let tx_hash = body.tx_hash.trim();
    let from_address = body.from_address.trim();

    if !tx_hash.starts_with("0x") || tx_hash.len() != 66 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Invalid tx_hash"}));
    }
    if !from_address.starts_with("0x") || from_address.len() != 42 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Invalid from_address"}));
    }

    let rows = match sqlx::query(
        r#"UPDATE payments
           SET tx_hash = $1, from_address = $2, status = 'broadcast', submitted_at = NOW()
           WHERE id = $3 AND payer_id = $4 AND status = 'pending'"#,
    )
    .bind(tx_hash)
    .bind(from_address)
    .bind(payment_id)
    .bind(claims.sub)
    .execute(pool.get_ref())
    .await
    {
        Ok(r) => r.rows_affected(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("payments_tx_hash_uq") {
                return HttpResponse::Conflict().json(serde_json::json!({
                    "error": "tx_hash already used by another payment"
                }));
            }
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": msg}));
        }
    };
    if rows == 0 {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Payment not found or not in pending state"
        }));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "payment_id": payment_id,
        "status": "broadcast",
    }))
}
