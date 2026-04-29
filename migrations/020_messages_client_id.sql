-- Add a client-generated idempotency key to messages so retried sends
-- (network blips on slow Lagos cellular) don't duplicate. The send handler
-- looks up (conversation_id, sender_id, client_id) and returns the
-- existing row instead of inserting a second one.
--
-- Nullable + no default because legacy clients still send without it; new
-- clients always populate it.

ALTER TABLE messages ADD COLUMN client_id TEXT;

-- Partial unique index: enforce uniqueness only when client_id is set, so
-- the millions of legacy NULL rows don't conflict with each other.
CREATE UNIQUE INDEX idx_messages_dedupe_client_id
    ON messages (conversation_id, sender_id, client_id)
    WHERE client_id IS NOT NULL;
