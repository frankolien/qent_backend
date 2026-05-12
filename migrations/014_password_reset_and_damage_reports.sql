-- Password reset tokens
CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    token VARCHAR(64) NOT NULL UNIQUE,
    expires_at TIMESTAMP NOT NULL,
    used BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Damage reports for return checklist
CREATE TABLE damage_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    booking_id UUID NOT NULL REFERENCES bookings(id),
    reporter_id UUID NOT NULL REFERENCES users(id),
    reporter_role VARCHAR(10) NOT NULL, -- 'host' or 'renter'
    photos TEXT[] NOT NULL DEFAULT '{}',
    notes TEXT,
    odometer_reading INTEGER,
    fuel_level VARCHAR(20), -- 'full', '3/4', '1/2', '1/4', 'empty'
    exterior_condition VARCHAR(20) NOT NULL DEFAULT 'good', -- 'good', 'minor_damage', 'major_damage'
    interior_condition VARCHAR(20) NOT NULL DEFAULT 'good',
    confirmed BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Add payout approval fields to wallet_transactions
ALTER TABLE wallet_transactions ADD COLUMN IF NOT EXISTS status VARCHAR(20) NOT NULL DEFAULT 'completed';
ALTER TABLE wallet_transactions ADD COLUMN IF NOT EXISTS admin_notes TEXT;

-- Add refresh_token column to users
ALTER TABLE users ADD COLUMN IF NOT EXISTS refresh_token VARCHAR(128);
