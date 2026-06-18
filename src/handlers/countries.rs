//! Country picker endpoints.
//!
//! V2 §6.3 — after the Privy sign-in, the mobile client lands on a
//! country picker. The list comes from the `countries` table (seeded
//! in migration 024); supported markets sort first, waitlist markets
//! render as "coming soon."
//!
//! `POST /api/users/me/country` sets the column on the `users` row.
//! It rejects unknown ISO codes via the existing FK rather than its
//! own lookup, so the picker can never store a value the rest of the
//! app can't interpret.

use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use utoipa::ToSchema;

use crate::models::Claims;

#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct CountryRow {
    pub iso2: String,
    pub name: String,
    pub currency_code: String,
    pub phone_code: String,
    pub supported: bool,
    pub plate_format_hint: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/countries",
    tag = "Countries",
    responses(
        (status = 200, description = "All known countries; supported sort first", body = Vec<CountryRow>),
    ),
)]
pub async fn list_countries(pool: web::Data<PgPool>) -> HttpResponse {
    let rows = sqlx::query_as::<_, CountryRow>(
        r#"SELECT iso2, name, currency_code, phone_code, supported, plate_format_hint
           FROM countries
           ORDER BY supported DESC, name ASC"#,
    )
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => {
            log::error!("list_countries failed: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({"error": "DB error"}))
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetCountryRequest {
    pub iso2: String,
}

#[utoipa::path(
    post,
    path = "/api/users/me/country",
    tag = "Users",
    security(("bearer_auth" = [])),
    request_body = SetCountryRequest,
    responses(
        (status = 200, description = "Country saved"),
        (status = 400, description = "Unsupported or unknown country"),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn set_my_country(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<SetCountryRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let iso = body.iso2.trim().to_uppercase();
    if iso.len() != 2 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "iso2 must be a 2-letter country code"}));
    }

    // Reject waitlist countries at the handler. The FK would accept
    // them, but §6.3 disallows landing in onboarding with an
    // unsupported country.
    let supported = sqlx::query_scalar::<_, bool>(
        "SELECT supported FROM countries WHERE iso2 = $1",
    )
    .bind(&iso)
    .fetch_optional(pool.get_ref())
    .await
    .ok()
    .flatten();

    let supported = match supported {
        Some(s) => s,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "Unknown country"}));
        }
    };

    if !supported {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Country not yet supported — join the waitlist"}));
    }

    let result = sqlx::query("UPDATE users SET country = $1, updated_at = NOW() WHERE id = $2")
        .bind(&iso)
        .bind(claims.sub)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"country": iso})),
        Err(e) => {
            log::error!("set_my_country failed: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({"error": "DB error"}))
        }
    }
}
