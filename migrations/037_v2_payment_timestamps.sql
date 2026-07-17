-- V2 §4.1 Phase 3 — track when a payment was broadcast vs. confirmed
-- so the `reconcile_chain_payments` Tokio job (§4.3 step 15) can pick
-- the right slice: payments still in `broadcast` for more than 30s.
--
-- Both are nullable: `submitted_at` is set when /submit-tx fires,
-- `confirmed_at` when the Alchemy webhook (or reconciler) flips
-- status to `success`. Backfill is unnecessary — no V2 payments
-- exist yet at this point.

ALTER TABLE payments
    ADD COLUMN IF NOT EXISTS submitted_at TIMESTAMP,
    ADD COLUMN IF NOT EXISTS confirmed_at TIMESTAMP;

-- Index used by the reconciler's pickup query. Partial keeps the
-- index tiny — only rows the job actually scans.
CREATE INDEX IF NOT EXISTS idx_payments_broadcast_age
    ON payments (submitted_at)
    WHERE status = 'broadcast';
