-- Saved payment cards (Paystack authorization tokens)
CREATE TABLE IF NOT EXISTS saved_cards (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    authorization_code VARCHAR(255) NOT NULL,
    card_type VARCHAR(50) NOT NULL,       -- visa, mastercard, etc.
    last4 VARCHAR(4) NOT NULL,
    exp_month VARCHAR(2) NOT NULL,
    exp_year VARCHAR(4) NOT NULL,
    bin VARCHAR(10) NOT NULL,             -- first 6 digits
    bank VARCHAR(100),
    brand VARCHAR(50) NOT NULL,           -- Visa, Mastercard, etc.
    is_default BOOLEAN NOT NULL DEFAULT false,
    cardholder_name VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_saved_cards_user_id ON saved_cards(user_id);

-- Ensure only one default card per user
CREATE UNIQUE INDEX IF NOT EXISTS idx_saved_cards_default
    ON saved_cards(user_id) WHERE is_default = true;
