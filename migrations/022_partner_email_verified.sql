-- Track whether a partner's contract email has been confirmed via the
-- existing 4-digit Resend OTP (verification_codes table). The Editorial
-- onboarding flow gates Step 02 (Vehicle) on this flag — once true, the
-- partner doesn't have to verify the same email again on retry.
ALTER TABLE partner_profiles
    ADD COLUMN IF NOT EXISTS contract_email_verified BOOLEAN NOT NULL DEFAULT FALSE;

CREATE INDEX IF NOT EXISTS partner_profiles_contract_email_verified_idx
    ON partner_profiles (contract_email_verified)
    WHERE contract_email_verified = TRUE;
