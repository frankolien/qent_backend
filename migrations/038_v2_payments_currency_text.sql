-- V2 §9.2 — payments.currency was varchar(3) for V1 fiat codes
-- (NGN, USD). V2 uses 'USDC' which is 4 chars. Widen to TEXT.
-- No data change; existing V1 rows fit fine.

ALTER TABLE payments
    ALTER COLUMN currency TYPE TEXT;
