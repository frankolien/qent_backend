-- Sign in with Google support
-- Adds google_id to users (Google's stable `sub` claim, unique per user)

ALTER TABLE users
    ADD COLUMN google_id TEXT UNIQUE;

CREATE INDEX idx_users_google_id ON users(google_id);
