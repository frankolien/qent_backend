//! V2 cars search.
//!
//! V2 §7 — minimum filter set for Week 1: country (required), city,
//! dates, USDC price range. The full filter sheet (§7.7) lands in
//! Week 5 — transmission, seats, fuel, instant-book, delivery,
//! distance-from-here. Keeping Week 1 minimal so the search endpoint
//! has a stable shape the mobile home view can wire against today.
//!
//! Date-based availability filters out cars that already have a
//! confirmed booking overlapping the requested window. Rejected /
//! cancelled bookings don't block.

use actix_web::{web, HttpResponse};
use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct CarSearchV2Query {
    pub country: String,
    pub city: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub min_price_usdc: Option<f64>,
    pub max_price_usdc: Option<f64>,
    pub sort_by: Option<String>, // price_asc | price_desc | rating | newest
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct CarSearchHit {
    pub id: Uuid,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub photos: Vec<String>,
    pub country: Option<String>,
    pub city: String,
    pub location: String,
    #[schema(value_type = String)]
    pub price_per_day_usdc: Decimal,
    pub instant_book: bool,
    pub min_trip_days: i32,
    pub rating: Option<f64>,
    pub trip_count: Option<i64>,
    pub host_name: Option<String>,
    pub created_at: NaiveDateTime,
}

#[utoipa::path(
    get,
    path = "/api/v2/cars/search",
    tag = "Cars",
    params(CarSearchV2Query),
    responses(
        (status = 200, description = "Paginated cars matching the V2 filter set", body = Vec<CarSearchHit>),
    ),
)]
pub async fn search(
    pool: web::Data<PgPool>,
    query: web::Query<CarSearchV2Query>,
) -> HttpResponse {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let country = query.country.trim().to_uppercase();
    if country.len() != 2 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "country must be a 2-letter ISO code"}));
    }

    let order = match query.sort_by.as_deref() {
        Some("price_asc") => "c.price_per_day_usdc ASC, c.created_at DESC",
        Some("price_desc") => "c.price_per_day_usdc DESC, c.created_at DESC",
        Some("newest") => "c.created_at DESC",
        _ => "COALESCE(rs.avg_rating, 0.0) DESC, c.created_at DESC",
    };

    let sql = format!(
        r#"SELECT
              c.id, c.make, c.model, c.year, c.photos,
              c.country, c.city, c.location,
              c.price_per_day_usdc, c.instant_book, c.min_trip_days,
              COALESCE(rs.avg_rating, 0.0) as rating,
              COALESCE(rs.trip_count, 0)   as trip_count,
              u.full_name as host_name,
              c.created_at
            FROM cars c
            LEFT JOIN (
                SELECT b.car_id,
                       AVG(r.rating)::double precision as avg_rating,
                       COUNT(DISTINCT b.id)           as trip_count
                FROM reviews r
                JOIN bookings b ON r.booking_id = b.id
                GROUP BY b.car_id
            ) rs ON rs.car_id = c.id
            LEFT JOIN users u ON u.id = c.host_id
            WHERE c.status = 'active'
              AND c.country = $1
              AND ($2::text IS NULL OR LOWER(c.city) = LOWER($2))
              AND ($3::numeric IS NULL OR c.price_per_day_usdc >= $3)
              AND ($4::numeric IS NULL OR c.price_per_day_usdc <= $4)
              -- Availability: exclude cars with a confirmed booking
              -- whose [start,end] overlaps the requested window.
              AND NOT EXISTS (
                  SELECT 1 FROM bookings b2
                  WHERE b2.car_id = c.id
                    AND b2.status IN ('paid','active','confirmed','approved','in_progress')
                    AND ($5::date IS NULL OR $6::date IS NULL OR
                         (b2.start_date <= $6 AND b2.end_date >= $5))
              )
              -- Host availability window (V1 columns) still respected.
              AND ($5::date IS NULL OR c.available_from IS NULL OR c.available_from <= $5)
              AND ($6::date IS NULL OR c.available_to   IS NULL OR c.available_to   >= $6)
            ORDER BY {order}
            LIMIT $7 OFFSET $8"#,
    );

    let min = query
        .min_price_usdc
        .and_then(|v| Decimal::from_f64_retain(v));
    let max = query
        .max_price_usdc
        .and_then(|v| Decimal::from_f64_retain(v));

    let result = sqlx::query_as::<_, CarSearchHit>(&sql)
        .bind(&country)
        .bind(query.city.as_deref())
        .bind(min)
        .bind(max)
        .bind(query.start_date)
        .bind(query.end_date)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(rows) => HttpResponse::Ok().json(rows),
        Err(e) => {
            log::error!("cars_v2::search failed: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}
