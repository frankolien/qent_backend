-- V2 §9.2 — payments swaps Paystack fields for chain fields.
--
-- V1 used a generic `provider` + `provider_reference` shape; V2 is
-- chain-only, so the columns become `chain` + `tx_hash`. Source/dest
-- addresses are recorded so the webhook handler (§4.3 step 16) can
-- assert from/to match what we expect.
--
-- The status column is widened to the §5.3 state machine: pending,
-- broadcast, success, mismatch, failed, refunded.
--
-- `provider` and `provider_reference` stay in the table for now; the
-- Rust port will drop references to them, then a later migration
-- removes the columns.

ALTER TABLE payments
    ADD COLUMN IF NOT EXISTS chain         TEXT NOT NULL DEFAULT 'base'
        CHECK (chain IN ('base')),
    ADD COLUMN IF NOT EXISTS tx_hash       TEXT,
    ADD COLUMN IF NOT EXISTS from_address  TEXT,
    ADD COLUMN IF NOT EXISTS to_address    TEXT,
    ADD COLUMN IF NOT EXISTS amount_usdc   NUMERIC(20,6) NOT NULL DEFAULT 0;

-- Partial unique index: a tx_hash is unique when present (NULL during
-- pending). Enforces the §5.6 webhook-idempotency invariant
-- structurally — a duplicate webhook + reconciler race on the same
-- tx_hash cannot insert two `payments` rows.
CREATE UNIQUE INDEX IF NOT EXISTS payments_tx_hash_uq
    ON payments(tx_hash) WHERE tx_hash IS NOT NULL;

CREATE INDEX IF NOT EXISTS payments_status_idx ON payments(status);

-- Migrate the status column to TEXT + CHECK per §9.2.
ALTER TABLE payments
    ALTER COLUMN status DROP DEFAULT;

ALTER TABLE payments
    ALTER COLUMN status TYPE TEXT USING status::TEXT;

ALTER TABLE payments
    ALTER COLUMN status SET DEFAULT 'pending';

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE table_name = 'payments' AND constraint_name = 'payments_status_check'
    ) THEN
        ALTER TABLE payments DROP CONSTRAINT payments_status_check;
    END IF;
END $$;

ALTER TABLE payments
    ADD CONSTRAINT payments_status_check CHECK (status IN (
        'pending',
        'broadcast',
        'success',
        'mismatch',
        'failed',
        'refunded',
        -- legacy V1 values; tolerated during transition.
        'completed', 'cancelled'
    ));
