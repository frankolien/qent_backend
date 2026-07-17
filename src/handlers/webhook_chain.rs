//! Alchemy webhook receiver — chain confirmations.
//!
//! V2 §4.1 steps 15-17, §4.3, §13.7.
//!
//! Discipline (§3.5, §4.3): verify HMAC signature first, then look up
//! `payments` by `tx_hash`, then assert from/to/amount match, then
//! transition the booking. Anything failing returns a clean 4xx so
//! Alchemy retries (per their docs, non-2xx triggers retry).
//!
//! Idempotency is structural: §9.2's `UNIQUE(tx_hash) WHERE NOT NULL`
//! partial index on `payments` makes the webhook + reconciler race a
//! no-op on the second arrival (the UPDATE either flips status or
//! finds the row already `success`).

use actix::Addr;
use actix_web::{web, HttpRequest, HttpResponse};
use hmac::{Hmac, Mac};
use rust_decimal::Decimal;
use serde::Deserialize;
use sha2::Sha256;
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

use crate::handlers::ws::{SendToUser, WsManager, WsMessage};
use crate::services::AppConfig;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize)]
struct AlchemyEnvelope {
    #[serde(rename = "type")]
    event_type: Option<String>,
    event: Option<AlchemyEvent>,
}

#[derive(Debug, Deserialize)]
struct AlchemyEvent {
    activity: Option<Vec<Activity>>,
}

#[derive(Debug, Deserialize)]
struct Activity {
    hash: Option<String>,
    #[serde(rename = "fromAddress")]
    from_address: Option<String>,
    #[serde(rename = "toAddress")]
    to_address: Option<String>,
    /// Alchemy emits the human-readable decimal amount here (raw /
    /// 10^decimals). Always parse as a string to dodge f64 precision.
    value: Option<serde_json::Value>,
    asset: Option<String>,
    category: Option<String>,
    #[serde(rename = "rawContract")]
    raw_contract: Option<RawContract>,
}

#[derive(Debug, Deserialize)]
struct RawContract {
    address: Option<String>,
}

pub async fn usdc_receive(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    ws_manager: web::Data<Addr<WsManager>>,
    body: web::Bytes,
) -> HttpResponse {
    // 1. HMAC verify. Reject with 401 (not 400) so Alchemy retries.
    if config.alchemy_webhook_secret.trim().is_empty() {
        log::error!("Alchemy webhook hit but ALCHEMY_WEBHOOK_SECRET not set");
        return HttpResponse::ServiceUnavailable().finish();
    }
    let signature = match req.headers().get("X-Alchemy-Signature").and_then(|v| v.to_str().ok()) {
        Some(s) => s,
        None => {
            log::warn!("Alchemy webhook missing X-Alchemy-Signature");
            return HttpResponse::Unauthorized().finish();
        }
    };
    let mut mac = match HmacSha256::new_from_slice(config.alchemy_webhook_secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    mac.update(&body);
    let expected = hex::encode(mac.finalize().into_bytes());
    if !constant_time_eq(signature.as_bytes(), expected.as_bytes()) {
        log::warn!("Alchemy webhook signature mismatch");
        return HttpResponse::Unauthorized().finish();
    }

    // 2. Parse.
    let envelope: AlchemyEnvelope = match serde_json::from_slice(&body) {
        Ok(e) => e,
        Err(e) => {
            log::warn!("Alchemy webhook bad JSON: {e}");
            return HttpResponse::BadRequest().finish();
        }
    };
    if envelope.event_type.as_deref() != Some("ADDRESS_ACTIVITY") {
        return HttpResponse::Ok().finish();
    }
    let activities = envelope
        .event
        .and_then(|e| e.activity)
        .unwrap_or_default();

    let usdc_contract = config.base_usdc_contract.to_lowercase();
    let escrow = config.escrow_wallet_address.to_lowercase();
    if escrow.is_empty() {
        log::error!("Alchemy webhook hit but escrow address not set");
        return HttpResponse::ServiceUnavailable().finish();
    }

    // 3. Process each activity. Per-activity failures don't fail the
    // whole batch — Alchemy bundles many transfers in one webhook.
    for a in activities {
        if let Err(e) = process_activity(&a, &usdc_contract, &escrow, &pool, &ws_manager).await {
            log::warn!("alchemy activity skipped: {e}");
        }
    }

    HttpResponse::Ok().finish()
}

#[derive(Debug, thiserror::Error)]
enum ActivityError {
    #[error("not a USDC token transfer")]
    NotUsdc,
    #[error("destination is not escrow")]
    NotEscrow,
    #[error("missing tx_hash")]
    MissingHash,
    #[error("invalid amount")]
    InvalidAmount,
    #[error("missing from_address")]
    MissingFrom,
    #[error("db error: {0}")]
    Db(#[from] sqlx::Error),
}

async fn process_activity(
    a: &Activity,
    usdc_contract: &str,
    escrow: &str,
    pool: &PgPool,
    ws_manager: &Addr<WsManager>,
) -> Result<(), ActivityError> {
    if a.category.as_deref() != Some("token") {
        return Err(ActivityError::NotUsdc);
    }
    let contract = a
        .raw_contract
        .as_ref()
        .and_then(|c| c.address.as_ref())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    if contract != usdc_contract {
        return Err(ActivityError::NotUsdc);
    }
    let to = a
        .to_address
        .as_ref()
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    if to != escrow {
        return Err(ActivityError::NotEscrow);
    }
    let tx_hash = a.hash.as_ref().ok_or(ActivityError::MissingHash)?.clone();
    let from = a
        .from_address
        .as_ref()
        .ok_or(ActivityError::MissingFrom)?
        .to_lowercase();
    let amount = parse_amount(a.value.as_ref()).ok_or(ActivityError::InvalidAmount)?;

    // Pull the matching payment + booking.
    let row = sqlx::query_as::<_, (Uuid, Uuid, String, Decimal, Option<String>, Uuid)>(
        r#"SELECT p.id, p.booking_id, p.status, p.amount_usdc::numeric, p.from_address, p.payer_id
           FROM payments p
           WHERE p.tx_hash = $1"#,
    )
    .bind(&tx_hash)
    .fetch_optional(pool)
    .await?;

    let Some((payment_id, booking_id, status, expected_amount, expected_from, payer_id)) = row
    else {
        // tx_hash not yet bound — the submit-tx race window. The
        // reconciler picks this up later by polling the chain.
        log::info!(
            "Alchemy webhook: no payment for tx_hash={} yet (race with submit-tx?)",
            tx_hash
        );
        return Ok(());
    };

    if status == "success" {
        // Idempotent — already confirmed.
        return Ok(());
    }
    if status != "broadcast" {
        log::warn!(
            "Alchemy webhook for payment {payment_id} in unexpected status {status} — ignoring"
        );
        return Ok(());
    }

    // Amount & from-address checks (§4.1 step 16). On mismatch → flag
    // the payment for admin review, don't auto-flip the booking.
    let amount_match = amount == expected_amount;
    let from_match = expected_from
        .as_ref()
        .map(|f| f.to_lowercase() == from)
        .unwrap_or(true); // if no from set, trust the chain

    if !amount_match || !from_match {
        sqlx::query(
            "UPDATE payments SET status = 'mismatch', confirmed_at = NOW() WHERE id = $1",
        )
        .bind(payment_id)
        .execute(pool)
        .await?;
        log::warn!(
            "Payment {payment_id} mismatch: amount {amount} vs {expected_amount}, from {from} vs {expected_from:?}"
        );
        return Ok(());
    }

    // Happy path — single transaction flips payment + booking.
    let mut tx = pool.begin().await?;
    sqlx::query(
        "UPDATE payments SET status = 'success', confirmed_at = NOW() WHERE id = $1 AND status = 'broadcast'",
    )
    .bind(payment_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE bookings SET status = 'paid', updated_at = NOW() WHERE id = $1 AND status = 'pending_payment'",
    )
    .bind(booking_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    // Fire-and-forget WS push (§4.1 step 17).
    ws_manager.do_send(SendToUser {
        user_id: payer_id,
        message: WsMessage {
            msg_type: "booking.paid".to_string(),
            payload: serde_json::json!({
                "booking_id": booking_id,
                "payment_id": payment_id,
                "tx_hash": tx_hash,
            }),
        },
    });

    log::info!("Booking {booking_id} marked paid (payment {payment_id}, tx {tx_hash})");
    Ok(())
}

fn parse_amount(v: Option<&serde_json::Value>) -> Option<Decimal> {
    let v = v?;
    match v {
        serde_json::Value::String(s) => Decimal::from_str(s.trim()).ok(),
        serde_json::Value::Number(n) => Decimal::from_str(&n.to_string()).ok(),
        _ => None,
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/webhooks/alchemy/usdc-receive",
        web::post().to(usdc_receive),
    );
}
