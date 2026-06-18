-- V2 §9.1, §5.4 — payouts table.
--
-- One row per outgoing transfer at booking completion. A single
-- booking generates 2-4 payouts (host_payout always; service_fee
-- always; protection_underwriter + protection_margin if protection
-- attached; refund/adjustment in dispute paths).
--
-- The state machine for each row is per §5.4:
--   queued → broadcast → success
--                     ↘ failed_retrying → broadcast | failed_manual
--
-- `tx_hash` is unique-when-present (same idempotency pattern as
-- payments) to make the §5.6 webhook-idempotency invariant
-- structurally enforced.
--
-- The partial index on status is for the payout job's pickup query:
-- it only needs to see rows that are not yet final.

CREATE TABLE IF NOT EXISTS payouts (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    booking_id           UUID NOT NULL REFERENCES bookings(id),
    kind                 TEXT NOT NULL CHECK (kind IN (
        'host_payout',
        'service_fee',
        'protection_underwriter',
        'protection_margin',
        'refund',
        'adjustment'
    )),
    amount_usdc          NUMERIC(20,6) NOT NULL,
    destination_address  TEXT NOT NULL,
    status               TEXT NOT NULL DEFAULT 'queued'
        CHECK (status IN (
            'queued',
            'broadcast',
            'success',
            'failed_retrying',
            'failed_manual'
        )),
    tx_hash              TEXT,
    chain                TEXT NOT NULL DEFAULT 'base'
        CHECK (chain IN ('base')),
    retry_count          INT NOT NULL DEFAULT 0,
    last_error           TEXT,
    created_at           TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS payouts_booking_idx ON payouts(booking_id);

-- Partial index for the payout job's pickup query.
CREATE INDEX IF NOT EXISTS payouts_status_active_idx
    ON payouts(status, created_at)
    WHERE status IN ('queued','broadcast','failed_retrying');

-- Structural idempotency for the webhook-and-reconciler race.
CREATE UNIQUE INDEX IF NOT EXISTS payouts_tx_hash_uq
    ON payouts(tx_hash) WHERE tx_hash IS NOT NULL;
