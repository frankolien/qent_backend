CREATE TABLE IF NOT EXISTS waitlist (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL,
    phone VARCHAR(30),
    name VARCHAR(100),
    role VARCHAR(20) NOT NULL DEFAULT 'renter', -- 'renter', 'host', 'both'
    city VARCHAR(100),
    referral_code VARCHAR(20),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(email)
);
