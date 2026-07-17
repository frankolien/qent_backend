//! Base L2 RPC + USDC transfer verification.
//!
//! Two responsibilities (V2 §3.3, §4.3):
//!   1. Verify an arbitrary tx_hash actually represents a USDC
//!      transfer from $from to $to of $amount, on the canonical
//!      native-USDC contract on Base. This is the structural check
//!      the Alchemy webhook handler and the
//!      `reconcile_chain_payments` job both call into.
//!   2. Query balances and recent activity on the platform escrow
//!      wallet — used by the §13.7 outflow watcher and the §5.6
//!      cross-machine-invariants nightly job.
//!
//! Calls go through Alchemy's Base RPC endpoint (auth via
//! API key in the URL). No private keys here — see `wallet.rs` for
//! transaction submission via Privy.
//!
//! Stub for Week 0; real RPC wiring lands in Week 2.

use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;

/// keccak256("Transfer(address,address,uint256)") — ERC-20 log topic.
const ERC20_TRANSFER_TOPIC: &str =
    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

#[derive(Debug, thiserror::Error)]
pub enum ChainError {
    #[error("Alchemy not configured")]
    NotConfigured,
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("tx_hash not found on chain")]
    TxNotFound,
    #[error("tx_hash present but reverted")]
    TxReverted,
    #[error("tx_hash present but not a USDC transfer matching expectation")]
    TxMismatch,
    #[error("token contract does not match canonical USDC on Base")]
    WrongTokenContract,
}

/// What we learn from a confirmed USDC transfer on Base. The
/// webhook handler asserts each field against the payment row.
#[derive(Debug, Clone)]
pub struct UsdcTransfer {
    pub tx_hash: String,
    pub from_address: String,
    pub to_address: String,
    pub amount_usdc: Decimal,
    pub confirmations: u32,
    pub block_number: u64,
}

#[derive(Clone)]
pub struct ChainClient {
    rpc_url: String,
    usdc_contract: String, // canonical native USDC on Base (§13.2)
    http: reqwest::Client,
}

impl ChainClient {
    pub fn new(rpc_url: String, usdc_contract: String) -> Self {
        Self {
            rpc_url,
            usdc_contract,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("reqwest client"),
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.rpc_url.trim().is_empty()
    }

    /// Fetch a USDC transfer by tx_hash. Returns `Err(TxNotFound)`
    /// if the tx isn't mined yet, `Err(TxMismatch)` if it's a
    /// different token / wrong shape, `Err(WrongTokenContract)` if
    /// it touches a non-canonical USDC.
    pub async fn fetch_usdc_transfer(&self, tx_hash: &str) -> Result<UsdcTransfer, ChainError> {
        if !self.is_configured() {
            return Err(ChainError::NotConfigured);
        }
        let receipt: Option<TxReceipt> = self
            .rpc_call("eth_getTransactionReceipt", serde_json::json!([tx_hash]))
            .await?;
        let Some(receipt) = receipt else {
            return Err(ChainError::TxNotFound);
        };
        if receipt.status.as_deref() != Some("0x1") {
            return Err(ChainError::TxReverted);
        }

        // Find the ERC-20 Transfer log on the canonical USDC contract.
        let usdc_contract = self.usdc_contract.to_lowercase();
        let log = receipt
            .logs
            .iter()
            .find(|l| {
                l.address.as_deref().map(|a| a.to_lowercase()).as_deref()
                    == Some(usdc_contract.as_str())
                    && l.topics
                        .first()
                        .map(|t| t.to_lowercase())
                        .as_deref()
                        == Some(ERC20_TRANSFER_TOPIC)
            })
            .ok_or(ChainError::TxMismatch)?;

        // topics[1] = padded from (32 bytes hex, last 20 are the address)
        // topics[2] = padded to
        let from = decode_topic_address(log.topics.get(1)).ok_or(ChainError::TxMismatch)?;
        let to = decode_topic_address(log.topics.get(2)).ok_or(ChainError::TxMismatch)?;
        let amount_raw = log.data.as_deref().ok_or(ChainError::TxMismatch)?;
        let amount = decode_uint256_to_usdc(amount_raw).ok_or(ChainError::TxMismatch)?;
        let block_number = hex_to_u64(&receipt.block_number).ok_or(ChainError::TxMismatch)?;

        // Confirmations require a follow-up `eth_blockNumber` call.
        // The reconciler doesn't need that level of precision; default
        // to 1 (mined) and let downstream logic gate on `status=success`.
        Ok(UsdcTransfer {
            tx_hash: tx_hash.to_string(),
            from_address: from,
            to_address: to,
            amount_usdc: amount,
            confirmations: 1,
            block_number,
        })
    }

    /// Current USDC balance of an arbitrary address on Base.
    /// Used by the §13.7 outflow watcher (deferred — not Week 2).
    pub async fn balance_of(&self, _address: &str) -> Result<Decimal, ChainError> {
        Err(ChainError::NotConfigured)
    }

    pub async fn is_block_reorged(&self, _block_number: u64) -> Result<bool, ChainError> {
        Ok(false)
    }

    async fn rpc_call<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T, ChainError> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });
        let resp: RpcResponse<T> = self
            .http
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        if let Some(err) = resp.error {
            return Err(ChainError::Rpc(err.message));
        }
        resp.result.ok_or(ChainError::TxNotFound)
    }
}

// ─── JSON-RPC envelope ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct TxReceipt {
    status: Option<String>,
    #[serde(rename = "blockNumber")]
    block_number: String,
    logs: Vec<Log>,
}

#[derive(Debug, Deserialize)]
struct Log {
    address: Option<String>,
    topics: Vec<String>,
    data: Option<String>,
}

// ─── Decoding helpers ──────────────────────────────────────────────────

fn decode_topic_address(topic: Option<&String>) -> Option<String> {
    let t = topic?.strip_prefix("0x")?;
    if t.len() < 40 {
        return None;
    }
    // Take the last 40 hex chars (20 bytes) as the address.
    let addr = &t[t.len() - 40..];
    Some(format!("0x{}", addr.to_lowercase()))
}

fn decode_uint256_to_usdc(data: &str) -> Option<Decimal> {
    // USDC has 6 decimals — raw = amount * 10^6.
    let trimmed = data.trim_start_matches("0x");
    // u128 holds up to ~3.4e38, USDC's max supply is well under 1e18 raw, fits.
    let raw = u128::from_str_radix(trimmed, 16).ok()?;
    Decimal::from_str(&raw.to_string()).ok().map(|d| d / Decimal::from(1_000_000u64))
}

fn hex_to_u64(s: &str) -> Option<u64> {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).ok()
}
