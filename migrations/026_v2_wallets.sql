-- V2 §9.1, §6.5 — wallets table.
--
-- One row per user, created behind the scenes during Phase 4 of
-- onboarding (Privy `users.create_wallet({ chain: "base" })`). The
-- separate table (rather than columns on `users`) is because the
-- wallet relationship may evolve — a user could link a second chain
-- later, a wallet provider could change, etc. Keeping it separate
-- keeps `users` simple.

CREATE TABLE IF NOT EXISTS wallets (
    user_id          UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    address          TEXT NOT NULL UNIQUE,
    chain            TEXT NOT NULL DEFAULT 'base'
        CHECK (chain IN ('base')),
    privy_wallet_id  TEXT NOT NULL UNIQUE,
    created_at       TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS wallets_address_idx ON wallets(address);
