//! New partner onboarding endpoints, backed by the `partner_profiles` and
//! `partner_listings` tables added in migration 021.
//!
//! Coexists with the legacy `partner.rs` (`/api/partner/apply`) which keeps
//! powering already-onboarded hosts in prod. New mobile clients hit these
//! endpoints; old clients keep working off the legacy single-shot flow
//! until we cut over completely.

use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::models::{
    Claims, CreatePartnerListingRequest, PartnerListing, PartnerProfile,
    SubmitOwnerConsentRequest, SubmitPartnerListingDocsRequest,
    UpdatePartnerListingPhotosRequest, UpsertPartnerProfileRequest,
};
use crate::services::prembly::PremblyClient;
use crate::services::AppConfig;

/// GET /api/partner/profile — Fetch the current user's identity profile.
/// Returns 404 if the user hasn't started onboarding yet — the mobile
/// client uses that to decide between "show Welcome" and "resume at the
/// step they were on".
#[utoipa::path(
    get,
    path = "/api/partner/profile",
    tag = "Partner v2",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Existing partner profile", body = PartnerProfile),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Onboarding not started"),
    ),
)]
pub async fn get_profile(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let result = sqlx::query_as::<_, PartnerProfile>(
        "SELECT * FROM partner_profiles WHERE user_id = $1",
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(p)) => HttpResponse::Ok().json(p),
        Ok(None) => HttpResponse::NotFound()
            .json(serde_json::json!({"error": "No partner profile yet"})),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()})),
    }
}

/// POST /api/partner/profile — Upsert the identity profile (step 02). The
/// host can come back to this screen and edit answers as long as we
/// haven't already verified them; once `identity_status = 'verified'`,
/// further self-service edits are blocked (they'd be a fraud vector —
/// silently swapping the legal name on a verified host).
#[utoipa::path(
    post,
    path = "/api/partner/profile",
    tag = "Partner v2",
    security(("bearer_auth" = [])),
    request_body = UpsertPartnerProfileRequest,
    responses(
        (status = 200, description = "Profile upserted", body = PartnerProfile),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "Profile already verified — contact support"),
    ),
)]
pub async fn upsert_profile(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<UpsertPartnerProfileRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({"errors": e.to_string()}));
    }

    // Lock out edits on already-verified profiles. Anything that needs
    // changing post-verification has to go through admin support so
    // the audit trail is preserved.
    let existing_status = sqlx::query_scalar::<_, String>(
        "SELECT identity_status FROM partner_profiles WHERE user_id = $1",
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await
    .ok()
    .flatten();

    if existing_status.as_deref() == Some("verified") {
        return HttpResponse::Conflict().json(serde_json::json!({
            "error": "Profile already verified — contact support to change identity details"
        }));
    }

    let result = sqlx::query_as::<_, PartnerProfile>(
        r#"INSERT INTO partner_profiles
            (user_id, profile_photo_url, legal_full_name, contract_email, phone,
             drivers_license_number, drivers_license_dob,
             drivers_license_front_url, drivers_license_back_url,
             identity_status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'draft')
        ON CONFLICT (user_id) DO UPDATE SET
            profile_photo_url = COALESCE(EXCLUDED.profile_photo_url, partner_profiles.profile_photo_url),
            legal_full_name = EXCLUDED.legal_full_name,
            contract_email = EXCLUDED.contract_email,
            phone = EXCLUDED.phone,
            drivers_license_number = EXCLUDED.drivers_license_number,
            drivers_license_dob = COALESCE(EXCLUDED.drivers_license_dob, partner_profiles.drivers_license_dob),
            drivers_license_front_url = COALESCE(EXCLUDED.drivers_license_front_url, partner_profiles.drivers_license_front_url),
            drivers_license_back_url = COALESCE(EXCLUDED.drivers_license_back_url, partner_profiles.drivers_license_back_url)
        RETURNING *"#,
    )
    .bind(claims.sub)
    .bind(&body.profile_photo_url)
    .bind(&body.legal_full_name)
    .bind(&body.contract_email)
    .bind(&body.phone)
    .bind(&body.drivers_license_number)
    .bind(body.drivers_license_dob)
    .bind(&body.drivers_license_front_url)
    .bind(&body.drivers_license_back_url)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(profile) => HttpResponse::Ok().json(profile),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()})),
    }
}

/// POST /api/partner/email/mark-verified — Called by the mobile after
/// the existing 4-digit Resend OTP (`/api/auth/verify-code`) returns
/// `verified: true`. We trust that response (it's already a bearer-
/// authed flow), match on the email currently on the partner profile,
/// and flip `contract_email_verified = true`. No new OTP table.
///
/// Idempotent — calling it twice with the same email is a no-op.
#[utoipa::path(
    post,
    path = "/api/partner/email/mark-verified",
    tag = "Partner v2",
    security(("bearer_auth" = [])),
    request_body = MarkEmailVerifiedRequest,
    responses(
        (status = 200, description = "Profile flagged as email-verified"),
        (status = 400, description = "Email doesn't match the profile on file"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "No profile yet — run Step 01 first"),
    ),
)]
pub async fn mark_email_verified(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<MarkEmailVerifiedRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let email = body.email.trim().to_lowercase();

    // Confirm the OTP actually fired and was marked verified for this
    // email. Don't take the client's word for it.
    let ok = sqlx::query_scalar::<_, bool>(
        r#"SELECT verified FROM verification_codes
           WHERE email = $1 AND verified = TRUE
           ORDER BY created_at DESC LIMIT 1"#,
    )
    .bind(&email)
    .fetch_optional(pool.get_ref())
    .await
    .ok()
    .flatten()
    .unwrap_or(false);

    if !ok {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Email not verified yet"}));
    }

    // Match against the email on the profile so a host can't flag a
    // *different* email as verified. Case-insensitive compare.
    let result = sqlx::query_scalar::<_, Uuid>(
        r#"UPDATE partner_profiles
           SET contract_email_verified = TRUE
           WHERE user_id = $1 AND LOWER(contract_email) = $2
           RETURNING id"#,
    )
    .bind(claims.sub)
    .bind(&email)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(_)) => HttpResponse::Ok().json(serde_json::json!({"verified": true})),
        Ok(None) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Email doesn't match the one on your profile"
        })),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()})),
    }
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct MarkEmailVerifiedRequest {
    pub email: String,
}

// ─── Listings ─────────────────────────────────────────────────────────────

/// GET /api/partner/listings — every listing the current user owns,
/// newest first. Powers both "Add new listing" (skip identity, jump
/// straight to vehicle for a fresh draft) and the host dashboard's
/// "your listings" section.
pub async fn list_listings(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let result = sqlx::query_as::<_, PartnerListing>(
        "SELECT * FROM partner_listings WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(claims.sub)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => HttpResponse::Ok().json(rows),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()})),
    }
}

/// GET /api/partner/listings/draft — the in-progress draft for resume.
/// At most one draft per user is meaningful; if there's somehow more
/// than one we return the newest.
pub async fn get_draft_listing(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let result = sqlx::query_as::<_, PartnerListing>(
        r#"SELECT * FROM partner_listings
           WHERE user_id = $1 AND listing_status = 'draft'
           ORDER BY created_at DESC LIMIT 1"#,
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(l)) => HttpResponse::Ok().json(l),
        Ok(None) => HttpResponse::NotFound()
            .json(serde_json::json!({"error": "No draft listing"})),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()})),
    }
}

/// POST /api/partner/listings — create a draft listing tied to the
/// caller's profile. Requires step 02 to have run (we need a profile
/// row first so foreign key holds).
///
/// Idempotent on the *current draft*: if there's already a draft for
/// this user we update its tier/brand/model/etc. in place rather than
/// stacking up multiple drafts. That matches the UX of "back, edit,
/// continue" which most hosts will do at least once.
pub async fn create_listing(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<CreatePartnerListingRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({"errors": e.to_string()}));
    }

    let profile_id_result = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM partner_profiles WHERE user_id = $1",
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await;

    let profile_id = match profile_id_result {
        Ok(Some(id)) => id,
        Ok(None) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Complete the Owner step first"
            }))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };

    let tier = body.tier.to_string();

    // Reuse the existing draft if any — saves the host from "I have 4
    // half-finished cars" weirdness when they hit back several times.
    let existing_draft = sqlx::query_scalar::<_, Uuid>(
        r#"SELECT id FROM partner_listings
           WHERE user_id = $1 AND listing_status = 'draft'
           ORDER BY created_at DESC LIMIT 1"#,
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await
    .ok()
    .flatten();

    let result = if let Some(draft_id) = existing_draft {
        sqlx::query_as::<_, PartnerListing>(
            r#"UPDATE partner_listings SET
                tier = $2, brand = $3, model = $4, year = $5,
                color = $6, plate_number = $7
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(draft_id)
        .bind(&tier)
        .bind(&body.brand)
        .bind(&body.model)
        .bind(body.year)
        .bind(&body.color)
        .bind(&body.plate_number)
        .fetch_one(pool.get_ref())
        .await
    } else {
        sqlx::query_as::<_, PartnerListing>(
            r#"INSERT INTO partner_listings
                (profile_id, user_id, tier, brand, model, year, color,
                 plate_number, listing_status, application_ref)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'draft', '')
               RETURNING *"#,
        )
        .bind(profile_id)
        .bind(claims.sub)
        .bind(&tier)
        .bind(&body.brand)
        .bind(&body.model)
        .bind(body.year)
        .bind(&body.color)
        .bind(&body.plate_number)
        .fetch_one(pool.get_ref())
        .await
    };

    match result {
        Ok(listing) => HttpResponse::Ok().json(listing),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()})),
    }
}

/// Minimum number of photos a listing must have before the host can
/// continue past Step 04. The mockup encourages 6 angles ("get booked
/// 3.4× faster"), but blocks at 3 so we don't wall them entirely on
/// first attempt — they can fill in extras during admin review.
const MIN_LISTING_PHOTOS: usize = 3;

/// PUT /api/partner/listings/{id}/photos — replace the photo array on
/// a draft listing. Order matters: index 0 is the cover/hero shot,
/// indexes 1..N are the angle slots in display order. The mobile
/// uploads each shot to Cloudinary first, so this endpoint just
/// stores URLs.
pub async fn update_listing_photos(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<UpdatePartnerListingPhotosRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let listing_id = path.into_inner();

    // Ownership + status guard combined: only mutate draft listings
    // that belong to the caller.
    let row = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT user_id, listing_status FROM partner_listings WHERE id = $1",
    )
    .bind(listing_id)
    .fetch_optional(pool.get_ref())
    .await;

    match row {
        Ok(Some((owner, status))) => {
            if owner != claims.sub {
                return HttpResponse::Forbidden()
                    .json(serde_json::json!({"error": "Not your listing"}));
            }
            if status != "draft" {
                return HttpResponse::Conflict().json(serde_json::json!({
                    "error": "Listing is no longer editable"
                }));
            }
        }
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": "Listing not found"}))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    }

    // Length cap: 7 slots in the UI; reject anything wildly out of
    // range so a buggy client can't blow up the row size.
    if body.photos.len() > 12 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Too many photos (max 12)"}));
    }

    let result = sqlx::query_as::<_, PartnerListing>(
        "UPDATE partner_listings SET photos = $2 WHERE id = $1 RETURNING *",
    )
    .bind(listing_id)
    .bind(&body.photos)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(listing) => {
            // Mirror the soft-min on the response so the client can
            // tell the host they're below the threshold without a
            // separate round-trip.
            HttpResponse::Ok().json(serde_json::json!({
                "listing": listing,
                "min_required": MIN_LISTING_PHOTOS,
                "meets_minimum": body.photos.len() >= MIN_LISTING_PHOTOS,
            }))
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()})),
    }
}

/// POST /api/partner/listings/{id}/docs — submit the document slice
/// (Step 05). Stores the URLs the host uploaded, runs Prembly DL +
/// plate verifications synchronously, and persists the structured
/// responses for audit. Insurance NIID isn't enabled on unverified
/// Prembly accounts, so we stash the certificate URL + policy number
/// without a live lookup for now.
///
/// Owner-mismatch is self-declared: the FRSC plate endpoint doesn't
/// expose the registered owner's name, so the client sends an
/// `is_registered_owner` boolean. `false` triggers the 04b branch.
pub async fn submit_listing_docs(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    path: web::Path<Uuid>,
    body: web::Json<SubmitPartnerListingDocsRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({"errors": e.to_string()}));
    }

    let listing_id = path.into_inner();

    // Ownership + status guard. Only mutate draft listings owned by
    // the caller — once submitted/approved, doc fields are locked.
    let listing_row = sqlx::query_as::<_, (Uuid, String, String, Uuid)>(
        "SELECT user_id, listing_status, plate_number, profile_id
         FROM partner_listings WHERE id = $1",
    )
    .bind(listing_id)
    .fetch_optional(pool.get_ref())
    .await;
    let (owner, status, plate_number, profile_id) = match listing_row {
        Ok(Some(t)) => t,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": "Listing not found"}))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };
    if owner != claims.sub {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Not your listing"}));
    }
    if status != "draft" {
        return HttpResponse::Conflict().json(serde_json::json!({
            "error": "Listing is no longer editable"
        }));
    }

    // Pull the profile so we know the host's full name + DL number to
    // forward to Prembly's DL verification.
    let profile = sqlx::query_as::<_, PartnerProfile>(
        "SELECT * FROM partner_profiles WHERE id = $1",
    )
    .bind(profile_id)
    .fetch_one(pool.get_ref())
    .await;
    let profile = match profile {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };

    let prembly = PremblyClient::new(
        config.prembly_secret_key.clone(),
        config.prembly_base_url.clone(),
    );

    // ─── DL verification ────────────────────────────────────────────────
    // Use the DOB the host typed on this screen if present; otherwise
    // fall back to whatever's already on the profile (rare — they
    // probably came back later to upload the licence images).
    let dl_dob = body
        .drivers_license_dob
        .or(profile.drivers_license_dob);
    let mut dl_verified = profile.drivers_license_frsc_verified;
    let mut dl_response: Option<serde_json::Value> = None;
    if prembly.is_configured() {
        if let Some(dob) = dl_dob {
            let (first, last) = split_full_name(&profile.legal_full_name);
            match prembly
                .verify_drivers_license(
                    &first,
                    &last,
                    &profile.drivers_license_number,
                    dob,
                )
                .await
            {
                Ok(out) => {
                    dl_verified = out.verified;
                    dl_response = Some(out.raw);
                }
                Err(e) => {
                    log::warn!("Prembly DL verify failed: {}", e);
                    dl_response = Some(serde_json::json!({"error": e.to_string()}));
                }
            }
        }
    }

    // Update the profile row with DL artefacts in one shot.
    let _ = sqlx::query(
        r#"UPDATE partner_profiles SET
            drivers_license_front_url = COALESCE($2, drivers_license_front_url),
            drivers_license_back_url  = COALESCE($3, drivers_license_back_url),
            drivers_license_dob       = COALESCE($4, drivers_license_dob),
            drivers_license_frsc_verified = $5,
            drivers_license_frsc_response = COALESCE($6, drivers_license_frsc_response)
           WHERE id = $1"#,
    )
    .bind(profile.id)
    .bind(&body.drivers_license_front_url)
    .bind(&body.drivers_license_back_url)
    .bind(dl_dob)
    .bind(dl_verified)
    .bind(dl_response.as_ref())
    .execute(pool.get_ref())
    .await;

    // ─── Plate verification ─────────────────────────────────────────────
    let mut plate_verified = false;
    let mut plate_response: Option<serde_json::Value> = None;
    if prembly.is_configured() {
        // Strip dashes/spaces — Prembly's regex requires letters then
        // numbers, no separators.
        let cleaned: String = plate_number
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect();
        match prembly.verify_vehicle_plate(&cleaned).await {
            Ok(out) => {
                plate_verified = out.verified;
                plate_response = Some(out.raw);
            }
            Err(e) => {
                log::warn!("Prembly plate verify failed: {}", e);
                plate_response = Some(serde_json::json!({"error": e.to_string()}));
            }
        }
    }

    // ─── Persist listing docs ───────────────────────────────────────────
    // owner_match is whatever the host declared — owner_consent_required
    // is its inverse so the 04b screen can be triggered without the
    // client having to negate it.
    let result = sqlx::query_as::<_, PartnerListing>(
        r#"UPDATE partner_listings SET
            vehicle_registration_url = COALESCE($2, vehicle_registration_url),
            vehicle_plate_frsc_verified = $3,
            vehicle_plate_frsc_response = COALESCE($4, vehicle_plate_frsc_response),
            insurance_certificate_url = COALESCE($5, insurance_certificate_url),
            insurance_policy_number  = COALESCE($6, insurance_policy_number),
            owner_match = $7,
            owner_consent_required = $8
           WHERE id = $1
           RETURNING *"#,
    )
    .bind(listing_id)
    .bind(&body.vehicle_registration_url)
    .bind(plate_verified)
    .bind(plate_response.as_ref())
    .bind(&body.insurance_certificate_url)
    .bind(&body.insurance_policy_number)
    .bind(body.is_registered_owner)
    .bind(!body.is_registered_owner)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(listing) => HttpResponse::Ok().json(serde_json::json!({
            "listing": listing,
            "drivers_license_verified": dl_verified,
            "vehicle_plate_verified": plate_verified,
            "owner_consent_required": !body.is_registered_owner,
        })),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()})),
    }
}

/// Cheap "first / last" split on the host's legal name. Anything past
/// the first whitespace is treated as the surname — Nigerian names
/// often have multiple given names so taking the *first* token alone
/// for first_name and the *rest* for last_name lines up with how FRSC
/// records most matches.
fn split_full_name(full: &str) -> (String, String) {
    let trimmed = full.trim();
    if let Some(idx) = trimmed.find(char::is_whitespace) {
        let (first, rest) = trimmed.split_at(idx);
        (first.trim().to_string(), rest.trim().to_string())
    } else {
        (trimmed.to_string(), trimmed.to_string())
    }
}

/// POST /api/partner/listings/{id}/owner-consent — Step 04b. Only
/// meaningful when the host declared they're NOT the registered
/// owner on the documents step (`owner_consent_required = true`).
/// Stores the relationship, the signed consent letter URL, the
/// owner's phone (for the eventual SMS OTP — which will be wired in
/// once we pick an SMS provider), and an optional owner NIN.
///
/// Refuses to apply when the listing is already past draft, or when
/// the docs step decided consent isn't needed (i.e. the host said
/// they're the owner — no consent letter to file).
pub async fn submit_owner_consent(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<SubmitOwnerConsentRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({"errors": e.to_string()}));
    }

    let listing_id = path.into_inner();

    // Ownership + status + consent-required guard.
    let row = sqlx::query_as::<_, (Uuid, String, bool)>(
        "SELECT user_id, listing_status, owner_consent_required
         FROM partner_listings WHERE id = $1",
    )
    .bind(listing_id)
    .fetch_optional(pool.get_ref())
    .await;
    let (owner, status, consent_required) = match row {
        Ok(Some(t)) => t,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": "Listing not found"}))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };
    if owner != claims.sub {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Not your listing"}));
    }
    if status != "draft" {
        return HttpResponse::Conflict().json(serde_json::json!({
            "error": "Listing is no longer editable"
        }));
    }
    if !consent_required {
        return HttpResponse::Conflict().json(serde_json::json!({
            "error": "Owner consent not required — toggle declared self as owner"
        }));
    }

    let result = sqlx::query_as::<_, PartnerListing>(
        r#"UPDATE partner_listings SET
            owner_relationship = $2,
            owner_consent_letter_url = COALESCE($3, owner_consent_letter_url),
            owner_consent_phone = $4,
            owner_nin = COALESCE($5, owner_nin)
           WHERE id = $1
           RETURNING *"#,
    )
    .bind(listing_id)
    .bind(&body.owner_relationship)
    .bind(&body.owner_consent_letter_url)
    .bind(&body.owner_consent_phone)
    .bind(&body.owner_nin)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(listing) => HttpResponse::Ok().json(serde_json::json!({"listing": listing})),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()})),
    }
}

/// POST /api/partner/identity/scan — Step 05 (Identity scan).
///
/// Stub for now. The real flow hands off to the Smile Identity SDK
/// for liveness + face match against the DL photo, and the SDK calls
/// us back with a job result. Until our Smile sandbox is approved we
/// store the selfie URL the host uploaded and mark the profile
/// `pending` so admin review can take it from there.
///
/// Schema fields are intentionally laid out the same as Smile's real
/// callback (smile_job_id, smile_response JSONB) so the swap to live
/// Smile is a one-method-body change, not a re-migration.
#[utoipa::path(
    post,
    path = "/api/partner/identity/scan",
    tag = "Partner v2",
    security(("bearer_auth" = [])),
    request_body = SubmitIdentityScanRequest,
    responses(
        (status = 200, description = "Profile updated; identity_status now pending"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "No profile yet — run Step 01 first"),
    ),
)]
pub async fn submit_identity_scan(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<SubmitIdentityScanRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    // Stand-in payload that mirrors what we'll eventually persist
    // from a real Smile callback. `mocked: true` is the giveaway when
    // looking at audit rows later.
    let mock_response = serde_json::json!({
        "mocked": true,
        "provider": "smile_identity",
        "note": "Sandbox-pending. Recorded for admin review.",
        "selfie_url": body.selfie_url,
    });

    let result = sqlx::query_as::<_, PartnerProfile>(
        r#"UPDATE partner_profiles SET
            selfie_url = $2,
            liveness_passed = true,
            face_match_score = 92.5,
            smile_response = $3,
            identity_status = CASE
                WHEN identity_status IN ('verified') THEN identity_status
                ELSE 'pending'
            END
           WHERE user_id = $1
           RETURNING *"#,
    )
    .bind(claims.sub)
    .bind(&body.selfie_url)
    .bind(&mock_response)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(profile)) => HttpResponse::Ok().json(serde_json::json!({
            "profile": profile,
            "verified": true, // mock — flip to real Smile decision later
            "score": 92.5,
        })),
        Ok(None) => HttpResponse::NotFound()
            .json(serde_json::json!({"error": "Run the Owner step first"})),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()})),
    }
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct SubmitIdentityScanRequest {
    /// Cloudinary URL of the selfie the host took on the Identity step.
    pub selfie_url: String,
}

/// POST /api/partner/listings/{id}/submit — Step 06 (Success).
///
/// Flips `listing_status` from `draft` → `submitted`, which makes the
/// row visible to the admin review queue. Validates every prerequisite
/// before stamping the status so we never end up with a "submitted"
/// listing that's half-empty.
///
/// Returns the listing including the auto-minted `application_ref`
/// (e.g. `QP-29841`) so the success screen can display it.
#[utoipa::path(
    post,
    path = "/api/partner/listings/{id}/submit",
    tag = "Partner v2",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Listing submitted; admin review will follow"),
        (status = 400, description = "Required fields missing"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not your listing"),
        (status = 404, description = "Listing not found"),
        (status = 409, description = "Already submitted"),
    ),
)]
pub async fn submit_listing(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let listing_id = path.into_inner();

    let row = sqlx::query_as::<_, PartnerListing>(
        "SELECT * FROM partner_listings WHERE id = $1",
    )
    .bind(listing_id)
    .fetch_optional(pool.get_ref())
    .await;
    let listing = match row {
        Ok(Some(l)) => l,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": "Listing not found"}))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };
    if listing.user_id != claims.sub {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Not your listing"}));
    }
    if listing.listing_status != "draft" {
        // Idempotent re-tap from the success screen — return the
        // existing row instead of yelling at the user.
        return HttpResponse::Ok().json(serde_json::json!({"listing": listing}));
    }

    // Hard prereqs — anything missing here means an earlier step
    // wasn't completed. Surface the offending field so the client can
    // tell the host where to go back to.
    if listing.photos.len() < MIN_LISTING_PHOTOS {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Add at least 3 photos before submitting",
            "field": "photos",
        }));
    }
    if listing
        .vehicle_registration_url
        .as_deref()
        .unwrap_or_default()
        .is_empty()
    {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Upload your vehicle registration first",
            "field": "vehicle_registration_url",
        }));
    }
    if listing.owner_consent_required
        && listing
            .owner_consent_letter_url
            .as_deref()
            .unwrap_or_default()
            .is_empty()
    {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Owner consent letter is required",
            "field": "owner_consent_letter_url",
        }));
    }

    let profile = sqlx::query_as::<_, PartnerProfile>(
        "SELECT * FROM partner_profiles WHERE id = $1",
    )
    .bind(listing.profile_id)
    .fetch_one(pool.get_ref())
    .await;
    let profile = match profile {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };
    if profile.selfie_url.as_deref().unwrap_or_default().is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Take the identity selfie first",
            "field": "selfie_url",
        }));
    }
    if !profile.contract_email_verified {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Verify your contract email first",
            "field": "contract_email_verified",
        }));
    }

    let result = sqlx::query_as::<_, PartnerListing>(
        "UPDATE partner_listings SET listing_status = 'submitted'
         WHERE id = $1 RETURNING *",
    )
    .bind(listing_id)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(l) => HttpResponse::Ok().json(serde_json::json!({"listing": l})),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()})),
    }
}
