-- V2 §13.5 — chargeback_holds.
--
-- Tracks delayed host payouts on first-card-funded bookings, per the
-- §13.5 launch policy: a renter who funds their wallet via card
-- on-ramp creates a chargeback window we have to outlive before we
-- release host funds.
--
-- The default V2 policy: 7-day hold post trip completion on the
-- *first* card-funded booking per renter; standard near-real-time
-- payout on subsequent bookings once the renter has one cleared
-- card-funded booking under their belt.
--
-- This table is the queue the payout job reads to know "is this
-- booking ready to release, or still in the chargeback window?"
-- The job releases when NOW() >= release_at AND chargeback_fired
-- is false.

CREATE TABLE IF NOT EXISTS chargeback_holds (
    booking_id        UUID PRIMARY KEY REFERENCES bookings(id),
    renter_id         UUID NOT NULL REFERENCES users(id),
    held_amount_usdc  NUMERIC(20,6) NOT NULL,
    onramp_partner    TEXT NOT NULL,         -- 'moonpay', 'yellow_card', ...
    onramp_reference  TEXT,                  -- partner's tx ref, for reconciliation
    release_at        TIMESTAMP NOT NULL,    -- usually trip_completion + 7 days
    chargeback_fired  BOOLEAN NOT NULL DEFAULT false,
    released          BOOLEAN NOT NULL DEFAULT false,
    released_at       TIMESTAMP,
    reason            TEXT,                  -- free text for the audit log
    created_at        TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS chargeback_holds_release_at_idx
    ON chargeback_holds(release_at)
    WHERE released = false AND chargeback_fired = false;

CREATE INDEX IF NOT EXISTS chargeback_holds_renter_idx
    ON chargeback_holds(renter_id);
