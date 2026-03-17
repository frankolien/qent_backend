-- Track user consent to terms of service and privacy policy
ALTER TABLE users ADD COLUMN IF NOT EXISTS tos_accepted_at TIMESTAMP;
ALTER TABLE users ADD COLUMN IF NOT EXISTS privacy_accepted_at TIMESTAMP;
ALTER TABLE users ADD COLUMN IF NOT EXISTS tos_version VARCHAR(10); -- e.g. '1.0', '1.1'

-- Account deletion requests (soft delete with grace period)
CREATE TABLE IF NOT EXISTS deletion_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    reason TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, processing, completed, cancelled
    requested_at TIMESTAMP NOT NULL DEFAULT NOW(),
    scheduled_deletion_at TIMESTAMP NOT NULL, -- 30 days from request
    completed_at TIMESTAMP
);

-- Audit log for sensitive actions (required by NDPA for accountability)
CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL, -- 'login', 'password_change', 'withdrawal', 'data_export', etc.
    ip_address VARCHAR(45),
    details JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Tax records for hosts (WHT deductions)
CREATE TABLE IF NOT EXISTS tax_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    booking_id UUID REFERENCES bookings(id),
    gross_amount DOUBLE PRECISION NOT NULL,
    platform_fee DOUBLE PRECISION NOT NULL,
    wht_amount DOUBLE PRECISION NOT NULL DEFAULT 0, -- Withholding tax (5% for individuals)
    net_payout DOUBLE PRECISION NOT NULL,
    tax_year INTEGER NOT NULL,
    tax_month INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_log_user ON audit_log(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_tax_records_user ON tax_records(user_id, tax_year, tax_month);
CREATE INDEX IF NOT EXISTS idx_deletion_requests_user ON deletion_requests(user_id);
