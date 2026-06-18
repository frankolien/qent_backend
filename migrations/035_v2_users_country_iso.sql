-- V2 §6.3 — make users.country an ISO-2 code that FKs into countries.
--
-- Migration 025 added `country TEXT REFERENCES countries(iso2)` with
-- IF NOT EXISTS — but the column already existed from 002 as
-- VARCHAR(100) NOT NULL DEFAULT 'Nigeria', so the FK and type change
-- silently didn't happen. This migration finishes the job:
--   1. Drop the NOT NULL default so Privy-onboarded users start with
--      country=NULL and pick it in the Phase 2 onboarding step.
--   2. Convert legacy full-name values ('Nigeria','Ghana',...) to
--      their ISO-2 codes so the FK can be added without nulls
--      elsewhere.
--   3. Add the FK to countries(iso2).
--
-- Safe to re-run because each step is idempotent (DROP DEFAULT,
-- UPDATE WHERE country IN (...), conditional constraint add).

ALTER TABLE users ALTER COLUMN country DROP NOT NULL;
ALTER TABLE users ALTER COLUMN country DROP DEFAULT;

UPDATE users SET country = 'NG' WHERE country = 'Nigeria';
UPDATE users SET country = 'GH' WHERE country = 'Ghana';
UPDATE users SET country = 'KE' WHERE country = 'Kenya';
UPDATE users SET country = 'ZA' WHERE country = 'South Africa';
UPDATE users SET country = 'AE' WHERE country IN ('UAE','United Arab Emirates');
UPDATE users SET country = 'GB' WHERE country IN ('UK','United Kingdom');
UPDATE users SET country = 'US' WHERE country IN ('USA','United States');
UPDATE users SET country = 'DE' WHERE country = 'Germany';
UPDATE users SET country = 'FR' WHERE country = 'France';

-- Anything still longer than 2 chars is unknown and gets nulled out
-- rather than blocking the FK. The country picker will catch it on
-- next sign-in.
UPDATE users SET country = NULL WHERE country IS NOT NULL AND LENGTH(country) <> 2;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE table_name = 'users' AND constraint_name = 'users_country_fkey'
    ) THEN
        ALTER TABLE users
            ADD CONSTRAINT users_country_fkey
            FOREIGN KEY (country) REFERENCES countries(iso2);
    END IF;
END $$;
