-- Refactor of the partner onboarding flow into two normalised tables:
--
--   partner_profiles  : 1 row per user — the identity / KYC bits that you
--                       only do ONCE per host (face match, DL, phone OTP,
--                       NIN, BVN). Every car listing references this.
--
--   partner_listings  : N rows per profile — vehicle-specific KYC + state
--                       (plate via FRSC, insurance via NIID, photos,
--                       owner-consent flow when the registered owner is
--                       not the host).
--
-- The old `partner_applications` table is left in place untouched —
-- existing prod hosts keep working off the legacy `/partner/apply`
-- endpoint until cutover. New onboarding writes to the new tables.

-- ─── partner_profiles ──────────────────────────────────────────────────────
-- One per user. Survives across multiple cars (Add new listing reuses it).
CREATE TABLE IF NOT EXISTS partner_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,

    -- Step 02 — Owner basics
    profile_photo_url TEXT,
    legal_full_name TEXT NOT NULL,
    contract_email TEXT NOT NULL,
    phone TEXT NOT NULL,
    phone_verified BOOLEAN NOT NULL DEFAULT false,

    -- Step 02 — Driver's license
    drivers_license_number TEXT NOT NULL,
    drivers_license_front_url TEXT,
    drivers_license_back_url TEXT,
    drivers_license_dob DATE,                 -- needed for FRSC lookup payload
    drivers_license_frsc_verified BOOLEAN NOT NULL DEFAULT false,
    drivers_license_frsc_response JSONB,      -- audit trail of Prembly call

    -- Optional belt-and-suspenders ID
    nin TEXT,
    nin_verified BOOLEAN NOT NULL DEFAULT false,
    nin_response JSONB,

    -- Step 06 — Identity scan (Smile Identity)
    selfie_url TEXT,
    liveness_passed BOOLEAN NOT NULL DEFAULT false,
    face_match_score REAL,                    -- 0-100 from Smile, NULL until ran
    smile_job_id TEXT,
    smile_response JSONB,

    -- Overall identity gate. When this hits 'verified' the profile is
    -- usable for any number of listings; rejecting kicks the host back
    -- to the relevant step. 'pending' means we're still waiting on a
    -- vendor or admin review.
    identity_status TEXT NOT NULL DEFAULT 'draft'
        CHECK (identity_status IN ('draft', 'pending', 'verified', 'rejected')),
    rejection_reason TEXT,

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_partner_profiles_status ON partner_profiles(identity_status);

-- ─── partner_listings ──────────────────────────────────────────────────────
-- One per car the host wants to list. Vehicle-specific KYC (FRSC plate
-- lookup, NIID insurance lookup, owner consent if the FRSC name doesn't
-- match the host) lives here. When approved we write the canonical row
-- into the existing `cars` table and link via `car_id`.
CREATE TABLE IF NOT EXISTS partner_listings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Human-readable application reference (e.g. QP-29841) for the
    -- success screen + admin emails. Generated from the sequence below.
    application_ref TEXT NOT NULL UNIQUE,
    profile_id UUID NOT NULL REFERENCES partner_profiles(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    car_id UUID REFERENCES cars(id),          -- set on approval

    -- Step 03 — Vehicle
    tier TEXT NOT NULL DEFAULT 'regular'
        CHECK (tier IN ('regular', 'luxury', 'exotic')),
    brand TEXT NOT NULL,
    model TEXT NOT NULL,
    year INT,
    color TEXT,
    plate_number TEXT NOT NULL,

    -- Step 04 — Photos. We let the photos count drift up to 7 in the
    -- mockup (1 cover + 6 angle slots); store as URL array, validate
    -- count at the API layer rather than the DB.
    photos TEXT[] NOT NULL DEFAULT '{}',

    -- Step 05 — Vehicle registration document (FRSC)
    vehicle_registration_url TEXT,
    vehicle_plate_frsc_verified BOOLEAN NOT NULL DEFAULT false,
    vehicle_plate_frsc_response JSONB,
    -- Owner name returned by FRSC plate lookup. Compared (fuzzy) against
    -- the profile's legal_full_name to decide whether the owner-consent
    -- branch fires.
    registered_owner_name TEXT,
    owner_match BOOLEAN,

    -- Step 05b — Owner consent (only when owner_match = false)
    owner_consent_required BOOLEAN NOT NULL DEFAULT false,
    owner_relationship TEXT,                  -- Father / Mother / Sibling / ...
    owner_consent_letter_url TEXT,
    owner_consent_phone TEXT,
    owner_consent_phone_verified BOOLEAN NOT NULL DEFAULT false,
    owner_consent_otp_code TEXT,              -- short-lived, cleared on verify
    owner_consent_otp_expires_at TIMESTAMP,
    owner_nin TEXT,
    owner_nin_verified BOOLEAN NOT NULL DEFAULT false,

    -- Step 05 — Insurance (NIID)
    insurance_certificate_url TEXT,
    insurance_policy_number TEXT,
    insurance_niid_verified BOOLEAN NOT NULL DEFAULT false,
    insurance_niid_response JSONB,

    -- Listing lifecycle. 'draft' = still being filled in by the host.
    -- 'submitted' = host hit Continue on the last step. 'in_review' =
    -- waiting on auto-decision rules / admin queue. 'approved' = car
    -- row created, listing is live (or scheduled live). 'rejected' =
    -- terminal failure with rejection_reason set.
    listing_status TEXT NOT NULL DEFAULT 'draft'
        CHECK (listing_status IN ('draft', 'submitted', 'in_review', 'approved', 'rejected')),
    rejection_reason TEXT,

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_partner_listings_profile ON partner_listings(profile_id);
CREATE INDEX idx_partner_listings_user ON partner_listings(user_id);
CREATE INDEX idx_partner_listings_status ON partner_listings(listing_status);
-- Plate is meant to be globally unique across LIVE listings, but during
-- onboarding two drafts could collide. Enforce uniqueness only on rows
-- past the draft stage so a host can re-edit without tripping the index.
CREATE UNIQUE INDEX idx_partner_listings_plate_active
    ON partner_listings(plate_number)
    WHERE listing_status IN ('submitted', 'in_review', 'approved');

-- Sequence + trigger to mint application_ref values like "QP-29841".
-- Sequencing in Postgres rather than the app code makes them
-- monotonically unique even under concurrent inserts.
CREATE SEQUENCE IF NOT EXISTS partner_listings_ref_seq START 10000;

CREATE OR REPLACE FUNCTION partner_listings_set_ref()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.application_ref IS NULL OR NEW.application_ref = '' THEN
        NEW.application_ref := 'QP-' || nextval('partner_listings_ref_seq');
    END IF;
    NEW.updated_at := NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS partner_listings_set_ref_trg ON partner_listings;
CREATE TRIGGER partner_listings_set_ref_trg
    BEFORE INSERT OR UPDATE ON partner_listings
    FOR EACH ROW
    EXECUTE FUNCTION partner_listings_set_ref();

-- partner_profiles also needs updated_at maintenance
CREATE OR REPLACE FUNCTION partner_profiles_touch()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at := NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS partner_profiles_touch_trg ON partner_profiles;
CREATE TRIGGER partner_profiles_touch_trg
    BEFORE UPDATE ON partner_profiles
    FOR EACH ROW
    EXECUTE FUNCTION partner_profiles_touch();
