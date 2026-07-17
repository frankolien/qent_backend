//! V2 §4.1 step 15 / §4.3 mitigation for "Alchemy webhook never fires."
//!
//! Every 60s, pick `payments` rows still in `broadcast` for more than
//! 30s and ask Base RPC directly whether the tx_hash settled. If it
//! did, flip the same transitions the webhook would have (payment →
//! `success`, booking → `paid`, WebSocket push). Idempotency-safe
//! against the webhook firing at the same moment: the UPDATE WHERE
//! status='broadcast' guard ensures only one path wins.

use actix::Addr;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

use crate::handlers::ws::{SendToUser, WsManager, WsMessage};
use crate::services::booking_workflows::BookingWorkflows;
use crate::services::chain::{ChainClient, ChainError};
use crate::services::AppConfig;

const POLL_INTERVAL: Duration = Duration::from_secs(60);
const MIN_BROADCAST_AGE_SECS: i64 = 30;
/// Cap the batch per tick so a backlog doesn't DoS Alchemy.
const MAX_BATCH: i64 = 50;

pub fn spawn(
    pool: PgPool,
    config: AppConfig,
    ws_manager: Addr<WsManager>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let chain = ChainClient::new(
            config.alchemy_rpc_url.clone(),
            config.base_usdc_contract.clone(),
        );
        let escrow = config.escrow_wallet_address.to_lowercase();

        if !chain.is_configured() {
            log::warn!("reconcile_chain_payments: ALCHEMY_RPC_URL not set — job idle");
        }
        if escrow.is_empty() {
            log::warn!("reconcile_chain_payments: ESCROW_WALLET_ADDRESS not set — job idle");
        }

        loop {
            tokio::time::sleep(POLL_INTERVAL).await;

            if !chain.is_configured() || escrow.is_empty() {
                continue;
            }

            if let Err(e) = tick(&pool, &chain, &escrow, &ws_manager).await {
                log::error!("reconcile_chain_payments tick error: {e}");
            }
        }
    })
}

async fn tick(
    pool: &PgPool,
    chain: &ChainClient,
    escrow: &str,
    ws_manager: &Addr<WsManager>,
) -> Result<(), sqlx::Error> {
    // Stragglers: status='broadcast' AND submitted_at < now - 30s.
    let candidates = sqlx::query_as::<_, (Uuid, Uuid, String, Decimal, Option<String>, Uuid)>(
        r#"SELECT p.id, p.booking_id, p.tx_hash, p.amount_usdc::numeric, p.from_address, p.payer_id
           FROM payments p
           WHERE p.status = 'broadcast'
             AND p.tx_hash IS NOT NULL
             AND p.submitted_at IS NOT NULL
             AND p.submitted_at < NOW() - ($1::text || ' seconds')::interval
           ORDER BY p.submitted_at ASC
           LIMIT $2"#,
    )
    .bind(MIN_BROADCAST_AGE_SECS.to_string())
    .bind(MAX_BATCH)
    .fetch_all(pool)
    .await?;

    if candidates.is_empty() {
        return Ok(());
    }
    log::info!(
        "reconcile_chain_payments: {} candidate(s)",
        candidates.len()
    );

    for (payment_id, booking_id, tx_hash, expected_amount, expected_from, payer_id) in candidates {
        match chain.fetch_usdc_transfer(&tx_hash).await {
            Ok(t) => {
                if t.to_address.to_lowercase() != escrow {
                    log::warn!(
                        "reconciler: payment {payment_id} tx {tx_hash} → wrong dest {}",
                        t.to_address
                    );
                    mark_mismatch(pool, payment_id).await?;
                    continue;
                }
                let amount_match = t.amount_usdc == expected_amount;
                let from_match = expected_from
                    .as_ref()
                    .map(|f| f.to_lowercase() == t.from_address.to_lowercase())
                    .unwrap_or(true);
                if !amount_match || !from_match {
                    log::warn!(
                        "reconciler: payment {payment_id} tx {tx_hash} mismatch (amount {} vs {expected_amount}, from {} vs {expected_from:?})",
                        t.amount_usdc,
                        t.from_address
                    );
                    mark_mismatch(pool, payment_id).await?;
                    continue;
                }
                let confirmation = BookingWorkflows::confirm_payment_received(
                    pool, payment_id, booking_id, payer_id,
                )
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
                if confirmation.changed {
                    ws_manager.do_send(SendToUser {
                        user_id: confirmation.payer_id,
                        message: WsMessage {
                            msg_type: "booking.paid".to_string(),
                            payload: serde_json::json!({
                                "booking_id": confirmation.booking_id,
                                "payment_id": confirmation.payment_id,
                                "tx_hash": tx_hash,
                                "via": "reconciler",
                            }),
                        },
                    });
                    log::info!(
                        "reconciler: booking {} marked paid via fallback path",
                        confirmation.booking_id
                    );
                }
            }
            Err(ChainError::TxNotFound) => {
                // Not mined yet — try again next tick.
            }
            Err(ChainError::TxReverted) => {
                log::warn!("reconciler: payment {payment_id} tx {tx_hash} reverted on chain");
                sqlx::query(
                    "UPDATE payments SET status = 'failed', confirmed_at = NOW() WHERE id = $1 AND status = 'broadcast'",
                )
                .bind(payment_id)
                .execute(pool)
                .await?;
            }
            Err(e) => {
                log::warn!("reconciler: payment {payment_id} chain lookup err: {e}");
            }
        }
    }
    Ok(())
}

async fn mark_mismatch(pool: &PgPool, payment_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE payments SET status = 'mismatch', confirmed_at = NOW() WHERE id = $1 AND status = 'broadcast'",
    )
    .bind(payment_id)
    .execute(pool)
    .await?;
    Ok(())
}
