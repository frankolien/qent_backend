-- V2 §6.9, §9.2 — users gets the V2 onboarding fields.
--
-- The five new columns mirror §6.9's table exactly. `country` FKs to
-- the new countries table (024) so country-driven config has a
-- single source of truth. `kyc_tier` replaces the V1
-- `verification_status` enum as the authoritative tier ladder
-- (§5.5); `verification_status` stays for now so V1 read paths keep
-- compiling.
--
-- `password_hash` becomes nullable because V2 users may authenticate
-- exclusively via Privy (Google / Apple / email-magic-link) and never
-- set a password.

ALTER TABLE users
    ALTER COLUMN password_hash DROP NOT NULL;

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS privy_user_id       TEXT UNIQUE,
    ADD COLUMN IF NOT EXISTS auth_provider       TEXT NOT NULL DEFAULT 'email'
        CHECK (auth_provider IN ('email','google','apple','privy')),
    ADD COLUMN IF NOT EXISTS country             TEXT REFERENCES countries(iso2),
    ADD COLUMN IF NOT EXISTS kyc_tier            INT NOT NULL DEFAULT 0
        CHECK (kyc_tier BETWEEN 0 AND 3),
    ADD COLUMN IF NOT EXISTS sumsub_applicant_id TEXT UNIQUE,
    ADD COLUMN IF NOT EXISTS phone_verified      BOOLEAN NOT NULL DEFAULT false;

CREATE INDEX IF NOT EXISTS users_country_idx  ON users(country);
CREATE INDEX IF NOT EXISTS users_kyc_tier_idx ON users(kyc_tier);
