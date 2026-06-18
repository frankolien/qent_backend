-- V2 §9.2 — bookings becomes USDC-denominated.
--
-- Six new columns capture the §2 worked-example math directly in the
-- row, plus a snapshot of the display currency + FX rate at the
-- moment of booking. The snapshot is the audit trail when a renter
-- complains three months later that "the rate was X when I booked."
--
-- The V1 columns (`subtotal`, `protection_fee`, `service_fee`,
-- `total_amount`) stay for now. They'll be dropped in a later
-- migration once the Rust handlers no longer reference them.

ALTER TABLE bookings
    ADD COLUMN IF NOT EXISTS host_price_usdc       NUMERIC(20,6) NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS service_fee_usdc      NUMERIC(20,6) NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS protection_price_usdc NUMERIC(20,6) NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS total_usdc            NUMERIC(20,6) NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS display_currency      TEXT NOT NULL DEFAULT 'NGN',
    ADD COLUMN IF NOT EXISTS display_fx_rate       NUMERIC(20,8) NOT NULL DEFAULT 1.0;

-- The cancellation_reason column is already there from 001; keep.

-- Two new terminal states from §5.2: no_show and failed_handover.
-- The status column today is a Postgres ENUM (booking_status); we
-- can't simply ADD VALUE inside a transaction in older Postgres,
-- but Postgres 12+ supports it. The transactional path is to add
-- the values; if the deploy uses a Postgres that lacks support,
-- this migration upgrades the column to TEXT + CHECK to match the
-- §9 schema. We pick TEXT + CHECK now because §9 commits to that
-- pattern for forward evolution (per the V1 book §16: enums are
-- harder to evolve than CHECK constraints).

ALTER TABLE bookings
    ALTER COLUMN status DROP DEFAULT;

ALTER TABLE bookings
    ALTER COLUMN status TYPE TEXT USING status::TEXT;

ALTER TABLE bookings
    ALTER COLUMN status SET DEFAULT 'pending_payment';

-- Drop any old CHECK constraint if it exists, then install the V2 one.
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE table_name = 'bookings' AND constraint_name = 'bookings_status_check'
    ) THEN
        ALTER TABLE bookings DROP CONSTRAINT bookings_status_check;
    END IF;
END $$;

ALTER TABLE bookings
    ADD CONSTRAINT bookings_status_check CHECK (status IN (
        'pending_payment',
        'paid',
        'active',
        'completed',
        'cancelled',
        'refunded',
        'disputed',
        'no_show',
        'failed_handover',
        -- legacy V1 enum values; tolerated during transition, removed
        -- in a later migration once handlers no longer emit them.
        'pending', 'confirmed', 'in_progress', 'approved', 'rejected'
    ));
