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

    /// Fetch a USDC transfer by tx_hash. Returns `Err(TxNotFound)`
    /// if the tx is not yet mined or does not exist.
    pub async fn fetch_usdc_transfer(&self, _tx_hash: &str) -> Result<UsdcTransfer, ChainError> {
        // Calls Alchemy `eth_getTransactionReceipt`, decodes the
        // USDC Transfer event, verifies the token contract matches
        // `self.usdc_contract`. Rejects bridged-USDC contracts.
        todo!("Week 2 — eth_getTransactionReceipt + USDC log decoding")
    }

    /// Current USDC balance of an arbitrary address on Base.
    /// Used by the §13.7 outflow watcher.
    pub async fn balance_of(&self, _address: &str) -> Result<Decimal, ChainError> {
        todo!("Week 2 — ERC-20 balanceOf via eth_call")
    }

    /// Did Alchemy report this block as part of a reorg? Used by
    /// the §13.1 reorg handler to downgrade `payments.success` back
    /// to `broadcast` if needed.
    pub async fn is_block_reorged(&self, _block_number: u64) -> Result<bool, ChainError> {
        todo!("Week 2 — query canonical chain for block hash, compare to recorded")
    }
}
