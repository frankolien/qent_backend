-- Sign in with Apple support
-- Adds apple_id to users (Apple's stable `sub` claim, unique per user per team)
-- Makes password_hash nullable so users who only ever sign in with Apple don't need one

ALTER TABLE users
    ADD COLUMN apple_id TEXT UNIQUE,
    ALTER COLUMN password_hash DROP NOT NULL;

CREATE INDEX idx_users_apple_id ON users(apple_id);
