-- V2 §6.10, §9.2 — partner_listings/profiles slimmed down.
--
-- Per §6.10, V2 deliberately stops collecting NIN / BVN / DL number /
-- NIID at onboarding time. Sumsub handles identity end-to-end; NIN
-- moves into the Sumsub flow as a country-specific add-on; BVN is
-- collected at withdrawal time by Yellow Card, not by us; NIID
-- (insurance certificate) is deferred indefinitely — the renter's
-- protection plan replaces it as the insurance vehicle.
--
-- We DROP rather than nullable-and-leave because the doc says we
-- never collect this data again, and leaving the columns nullable
-- invites a future handler to start populating them.
--
-- partner_profiles is the table that held identity-once-per-host
-- fields (NIN, DL); partner_listings held vehicle-per-listing fields
-- (NIID, plate FRSC). Both get trimmed.

ALTER TABLE partner_profiles
    DROP COLUMN IF EXISTS nin,
    DROP COLUMN IF EXISTS nin_verified,
    DROP COLUMN IF EXISTS nin_response,
    DROP COLUMN IF EXISTS drivers_license_number,
    DROP COLUMN IF EXISTS drivers_license_front_url,
    DROP COLUMN IF EXISTS drivers_license_back_url,
    DROP COLUMN IF EXISTS drivers_license_dob,
    DROP COLUMN IF EXISTS drivers_license_frsc_verified,
    DROP COLUMN IF EXISTS drivers_license_frsc_response,
    DROP COLUMN IF EXISTS selfie_url,
    DROP COLUMN IF EXISTS liveness_passed,
    DROP COLUMN IF EXISTS face_match_score,
    DROP COLUMN IF EXISTS smile_job_id,
    DROP COLUMN IF EXISTS smile_response;

-- partner_listings: drop NIID + plate-FRSC payload if present.
-- The 021 schema named these `niid_*` / `plate_frsc_*`. Defensive
-- IF EXISTS so the migration is safe across schema variants.
ALTER TABLE partner_listings
    DROP COLUMN IF EXISTS niid_url,
    DROP COLUMN IF EXISTS niid_number,
    DROP COLUMN IF EXISTS niid_verified,
    DROP COLUMN IF EXISTS niid_response,
    DROP COLUMN IF EXISTS plate_frsc_verified,
    DROP COLUMN IF EXISTS plate_frsc_response;

-- Two columns we keep on partner_listings:
--   vehicle_registration_url  — the V5 / reg doc photo (still collected at tier_2)
--   owner_consent_url         — "I own this car" letter (still collected at tier_2)
-- If 021 named them differently, the Rust port maps to whichever
-- exists. We don't rename here to keep this migration purely additive
-- on the drop side.
