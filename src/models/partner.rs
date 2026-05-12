use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PartnerApplicationStatus {
    Pending,
    Approved,
    Rejected,
}

impl std::fmt::Display for PartnerApplicationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartnerApplicationStatus::Pending => write!(f, "pending"),
            PartnerApplicationStatus::Approved => write!(f, "approved"),
            PartnerApplicationStatus::Rejected => write!(f, "rejected"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct PartnerApplication {
    pub id: Uuid,
    pub user_id: Uuid,
    pub full_name: String,
    pub email: String,
    pub phone: String,
    pub drivers_license: String,
    pub car_make: String,
    pub car_model: String,
    pub car_year: i32,
    pub car_color: String,
    pub car_plate_number: String,
    pub car_photos: Vec<String>,
    pub car_description: String,
    pub fuel_type: String,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreatePartnerApplicationRequest {
    #[validate(length(min = 2))]
    pub full_name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 5))]
    pub phone: String,
    #[validate(length(min = 1))]
    pub drivers_license: String,
    #[validate(length(min = 1))]
    pub car_make: String,
    #[validate(length(min = 1))]
    pub car_model: String,
    pub car_year: i32,
    #[validate(length(min = 1))]
    pub car_color: String,
    #[validate(length(min = 1))]
    pub car_plate_number: String,
    pub car_photos: Vec<String>,
    pub car_description: Option<String>,
    pub fuel_type: Option<String>,
    pub price_per_day: f64,
    pub location: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HostDashboard {
    pub total_earnings: f64,
    pub active_listings: i64,
    pub completed_bookings: i64,
    pub average_rating: f64,
}

// ─── New partner onboarding (migration 021) ────────────────────────────────
//
// `partner_profiles` is 1-per-user identity/KYC, `partner_listings` is N-per-
// profile vehicle KYC + lifecycle. The legacy `PartnerApplication` above
// stays in place so the old `/api/partner/apply` endpoint keeps compiling
// for hosts already onboarded under the previous flow.

/// High-level state of the host's identity verification. Drives whether
/// they can submit listings at all — once `verified`, any number of cars
/// can be added without re-doing the selfie / DL / phone steps.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PartnerIdentityStatus {
    /// Profile row exists but the user hasn't finished step 02 yet.
    Draft,
    /// Submitted, waiting on a vendor verdict or admin review.
    Pending,
    /// Identity confirmed; profile is reusable across listings.
    Verified,
    /// Hard fail. `rejection_reason` on the row explains why.
    Rejected,
}

impl std::fmt::Display for PartnerIdentityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartnerIdentityStatus::Draft => write!(f, "draft"),
            PartnerIdentityStatus::Pending => write!(f, "pending"),
            PartnerIdentityStatus::Verified => write!(f, "verified"),
            PartnerIdentityStatus::Rejected => write!(f, "rejected"),
        }
    }
}

/// Per-listing lifecycle.
///
///  - `Draft`     — host is still walking the steps, may abandon
///  - `Submitted` — host hit Continue on the last step
///  - `InReview`  — auto-decision rules pending or admin queue
///  - `Approved`  — `cars` row exists, listing is/about-to-be live
///  - `Rejected`  — terminal failure with `rejection_reason` set
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PartnerListingStatus {
    Draft,
    Submitted,
    InReview,
    Approved,
    Rejected,
}

impl std::fmt::Display for PartnerListingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartnerListingStatus::Draft => write!(f, "draft"),
            PartnerListingStatus::Submitted => write!(f, "submitted"),
            PartnerListingStatus::InReview => write!(f, "in_review"),
            PartnerListingStatus::Approved => write!(f, "approved"),
            PartnerListingStatus::Rejected => write!(f, "rejected"),
        }
    }
}

/// Listing tier — affects which brand list shows on step 03 and (later)
/// the protection plan + price-band defaults.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PartnerListingTier {
    Regular,
    Luxury,
    Exotic,
}

impl std::fmt::Display for PartnerListingTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartnerListingTier::Regular => write!(f, "regular"),
            PartnerListingTier::Luxury => write!(f, "luxury"),
            PartnerListingTier::Exotic => write!(f, "exotic"),
        }
    }
}

/// Row of `partner_profiles`. JSONB audit columns are surfaced as
/// `serde_json::Value` so we can pass them through to admin / debug
/// without modeling the full vendor response shape.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct PartnerProfile {
    pub id: Uuid,
    pub user_id: Uuid,

    // Owner basics (step 02)
    pub profile_photo_url: Option<String>,
    pub legal_full_name: String,
    pub contract_email: String,
    pub contract_email_verified: bool,
    pub phone: String,
    pub phone_verified: bool,

    // Driver's license
    pub drivers_license_number: String,
    pub drivers_license_front_url: Option<String>,
    pub drivers_license_back_url: Option<String>,
    pub drivers_license_dob: Option<chrono::NaiveDate>,
    pub drivers_license_frsc_verified: bool,
    #[schema(value_type = Object)]
    pub drivers_license_frsc_response: Option<serde_json::Value>,

    // Optional NIN
    pub nin: Option<String>,
    pub nin_verified: bool,
    #[schema(value_type = Object)]
    pub nin_response: Option<serde_json::Value>,

    // Identity scan (step 06)
    pub selfie_url: Option<String>,
    pub liveness_passed: bool,
    pub face_match_score: Option<f32>,
    pub smile_job_id: Option<String>,
    #[schema(value_type = Object)]
    pub smile_response: Option<serde_json::Value>,

    pub identity_status: String,
    pub rejection_reason: Option<String>,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Row of `partner_listings`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct PartnerListing {
    pub id: Uuid,
    pub application_ref: String,
    pub profile_id: Uuid,
    pub user_id: Uuid,
    pub car_id: Option<Uuid>,

    // Vehicle (step 03)
    pub tier: String,
    pub brand: String,
    pub model: String,
    pub year: Option<i32>,
    pub color: Option<String>,
    pub plate_number: String,

    // Photos (step 04)
    pub photos: Vec<String>,

    // Vehicle registration / FRSC (step 05)
    pub vehicle_registration_url: Option<String>,
    pub vehicle_plate_frsc_verified: bool,
    #[schema(value_type = Object)]
    pub vehicle_plate_frsc_response: Option<serde_json::Value>,
    pub registered_owner_name: Option<String>,
    pub owner_match: Option<bool>,

    // Owner consent (step 05b — conditional)
    pub owner_consent_required: bool,
    pub owner_relationship: Option<String>,
    pub owner_consent_letter_url: Option<String>,
    pub owner_consent_phone: Option<String>,
    pub owner_consent_phone_verified: bool,
    pub owner_consent_otp_code: Option<String>,
    pub owner_consent_otp_expires_at: Option<NaiveDateTime>,
    pub owner_nin: Option<String>,
    pub owner_nin_verified: bool,

    // Insurance / NIID (step 05)
    pub insurance_certificate_url: Option<String>,
    pub insurance_policy_number: Option<String>,
    pub insurance_niid_verified: bool,
    #[schema(value_type = Object)]
    pub insurance_niid_response: Option<serde_json::Value>,

    pub listing_status: String,
    pub rejection_reason: Option<String>,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// ─── Request bodies ────────────────────────────────────────────────────────

/// Step 02 — create / upsert the host profile. Idempotent per user: if a
/// profile already exists we update it in place, so the host can edit
/// their answers and re-submit.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpsertPartnerProfileRequest {
    #[validate(length(min = 2))]
    pub legal_full_name: String,
    #[validate(email)]
    pub contract_email: String,
    #[validate(length(min = 5))]
    pub phone: String,
    #[validate(length(min = 1))]
    pub drivers_license_number: String,
    pub drivers_license_dob: Option<chrono::NaiveDate>,
    pub profile_photo_url: Option<String>,
    pub drivers_license_front_url: Option<String>,
    pub drivers_license_back_url: Option<String>,
}

/// Step 03 — create a new listing draft for the current user. Profile
/// must exist (step 02 done).
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreatePartnerListingRequest {
    pub tier: PartnerListingTier,
    #[validate(length(min = 1))]
    pub brand: String,
    #[validate(length(min = 1))]
    pub model: String,
    pub year: Option<i32>,
    pub color: Option<String>,
    #[validate(length(min = 1))]
    pub plate_number: String,
}

/// Step 04 — replace the photo set on an existing listing. Validation
/// of count (≥ 3, ≤ 7) is done at the handler since it varies by tier.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePartnerListingPhotosRequest {
    pub photos: Vec<String>,
}

/// Step 05 — submit insurance + vehicle registration for verification.
/// Triggers Prembly DL + plate calls server-side; result is stored on
/// the profile / listing row. Insurance live-lookup is gated on a
/// production Prembly account, so for now we just store the URL and
/// policy number.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SubmitPartnerListingDocsRequest {
    /// Cloudinary URL of the host's driver's licence (front).
    pub drivers_license_front_url: Option<String>,
    /// Cloudinary URL of the host's driver's licence (back).
    pub drivers_license_back_url: Option<String>,
    /// Date of birth as printed on the licence (YYYY-MM-DD). Required
    /// by FRSC's verification payload.
    pub drivers_license_dob: Option<chrono::NaiveDate>,

    pub vehicle_registration_url: Option<String>,

    pub insurance_certificate_url: Option<String>,
    pub insurance_policy_number: Option<String>,

    /// Host's declaration that they're the registered owner of the
    /// vehicle on the C of R. Drives the conditional 04b consent
    /// branch — `false` means the client should push the owner
    /// consent screen next.
    pub is_registered_owner: bool,
}

/// Step 05b — owner-consent details, only sent if the FRSC plate lookup
/// returned a name that didn't match the host's legal name.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SubmitOwnerConsentRequest {
    #[validate(length(min = 1))]
    pub owner_relationship: String,
    pub owner_consent_letter_url: Option<String>,
    #[validate(length(min = 5))]
    pub owner_consent_phone: String,
    pub owner_nin: Option<String>,
}
