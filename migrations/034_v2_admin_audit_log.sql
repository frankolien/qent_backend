-- V2 §13.9 — admin_audit_log.
--
-- Money-integrity record, not a UI feature. Every manual money
-- action (refund issued, payout retried, KYC tier overridden,
-- account blocked, listing forced-approved/rejected, ESCROW_HALT
-- toggled, Privy credential rotated) writes one row here.
--
-- Append-only by Postgres role policy: the application user is
-- granted INSERT only. UPDATE and DELETE are revoked, so a bug in
-- the Rust handlers cannot retroactively edit history. A forensic
-- engineer can answer "who refunded what when" from this table
-- alone, with no application code running.
--
-- Retention: indefinite. Volume is low enough that pruning never
-- becomes necessary at our scale.

CREATE TABLE IF NOT EXISTS admin_audit_log (
    id              BIGSERIAL PRIMARY KEY,
    actor_user_id   UUID NOT NULL REFERENCES users(id),
    actor_role      TEXT NOT NULL,
    action          TEXT NOT NULL,
    entity_type     TEXT NOT NULL,
    entity_id       TEXT NOT NULL,
    before_state    JSONB,
    after_state     JSONB,
    reason          TEXT,
    ip_address      INET,
    created_at      TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS admin_audit_log_entity_idx
    ON admin_audit_log(entity_type, entity_id, created_at DESC);

CREATE INDEX IF NOT EXISTS admin_audit_log_actor_idx
    ON admin_audit_log(actor_user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS admin_audit_log_action_idx
    ON admin_audit_log(action, created_at DESC);

-- Append-only enforcement at the role level. This runs only if the
-- application role exists; in local dev the app may connect as the
-- DB owner which is exempt by design. Production deploys should
-- create a dedicated qent_app role and grant only INSERT here.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'qent_app') THEN
        REVOKE UPDATE, DELETE ON admin_audit_log FROM qent_app;
        GRANT  INSERT, SELECT  ON admin_audit_log TO qent_app;
        GRANT  USAGE, SELECT   ON SEQUENCE admin_audit_log_id_seq TO qent_app;
    END IF;
END $$;
