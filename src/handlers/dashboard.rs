use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::Claims;

#[derive(Debug, Serialize)]
pub struct HostStats {
    pub total_listings: i64,
    pub active_listings: i64,
    pub total_views: i64,
    pub total_bookings: i64,
    pub total_earnings: f64,
    pub this_month_earnings: f64,
    pub average_rating: f64,
    pub wallet_balance: f64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ListingSummary {
    pub id: Uuid,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub photo: String,
    pub price_per_day: f64,
    pub status: String,
    pub views_count: i32,
    pub rating: f64,
    pub trip_count: i64,
}

/// GET /api/dashboard/stats - Host dashboard statistics
pub async fn get_host_stats(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let host_id = claims.sub;

    // Fetch all stats in parallel queries
    let total_listings =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM cars WHERE host_id = $1")
            .bind(host_id)
            .fetch_one(pool.get_ref())
            .await
            .unwrap_or(0);

    let active_listings = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM cars WHERE host_id = $1 AND status = 'active'",
    )
    .bind(host_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);

    let total_views = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(SUM(views_count::bigint), 0) FROM cars WHERE host_id = $1",
    )
    .bind(host_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);

    let total_bookings = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM bookings b
        JOIN cars c ON c.id = b.car_id
        WHERE c.host_id = $1"#,
    )
    .bind(host_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);

    let total_earnings = sqlx::query_scalar::<_, f64>(
        r#"SELECT COALESCE(SUM(b.total_price), 0.0)
        FROM bookings b
        JOIN cars c ON c.id = b.car_id
        WHERE c.host_id = $1 AND b.status = 'completed'"#,
    )
    .bind(host_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0.0);

    let this_month_earnings = sqlx::query_scalar::<_, f64>(
        r#"SELECT COALESCE(SUM(b.total_price), 0.0)
        FROM bookings b
        JOIN cars c ON c.id = b.car_id
        WHERE c.host_id = $1 AND b.status = 'completed'
        AND b.created_at >= date_trunc('month', CURRENT_DATE)"#,
    )
    .bind(host_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0.0);

    let average_rating = sqlx::query_scalar::<_, f64>(
        r#"SELECT COALESCE(AVG(r.rating), 0.0)::double precision
        FROM reviews r
        JOIN bookings b ON b.id = r.booking_id
        JOIN cars c ON c.id = b.car_id
        WHERE c.host_id = $1"#,
    )
    .bind(host_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0.0);

    let wallet_balance = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(wallet_balance, 0.0) FROM users WHERE id = $1",
    )
    .bind(host_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0.0);

    HttpResponse::Ok().json(HostStats {
        total_listings,
        active_listings,
        total_views,
        total_bookings,
        total_earnings,
        this_month_earnings,
        average_rating,
        wallet_balance,
    })
}

/// GET /api/dashboard/listings - Host's car listings with stats
pub async fn get_host_listings(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let result = sqlx::query_as::<_, ListingSummary>(
        r#"SELECT
            c.id, c.make, c.model, c.year,
            COALESCE(c.photos[1], '') as photo,
            c.price_per_day,
            c.status::text,
            c.views_count,
            COALESCE(rs.avg_rating, 0.0)::double precision as rating,
            COALESCE(rs.trip_count, 0) as trip_count
        FROM cars c
        LEFT JOIN (
            SELECT b.car_id,
                   AVG(r.rating)::double precision as avg_rating,
                   COUNT(DISTINCT b.id) as trip_count
            FROM reviews r
            JOIN bookings b ON r.booking_id = b.id
            GROUP BY b.car_id
        ) rs ON rs.car_id = c.id
        WHERE c.host_id = $1
        ORDER BY c.created_at DESC"#,
    )
    .bind(claims.sub)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(listings) => HttpResponse::Ok().json(listings),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// POST /api/cars/{id}/view - Increment view count
pub async fn increment_view(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let car_id = path.into_inner();

    let _ = sqlx::query("UPDATE cars SET views_count = views_count + 1 WHERE id = $1")
        .bind(car_id)
        .execute(pool.get_ref())
        .await;

    HttpResponse::Ok().json(serde_json::json!({"message": "View recorded"}))
}
