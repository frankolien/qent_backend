use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Claims, CreateDamageReportRequest, DamageReport};

/// POST /api/damage-reports — Submit a damage/return report for a booking
pub async fn create_report(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<CreateDamageReportRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    // Verify the booking exists and user is involved
    let booking = sqlx::query_as::<_, (Uuid, Uuid, String)>(
        "SELECT renter_id, host_id, status::text FROM bookings WHERE id = $1",
    )
    .bind(body.booking_id)
    .fetch_optional(pool.get_ref())
    .await;

    let (renter_id, host_id, status) = match booking {
        Ok(Some(b)) => b,
        _ => return HttpResponse::NotFound().json(serde_json::json!({"error": "Booking not found"})),
    };

    if claims.sub != renter_id && claims.sub != host_id {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": "Not your booking"}));
    }

    if status != "active" && status != "completed" {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Can only submit reports for active or completed trips"}));
    }

    let role = if claims.sub == host_id { "host" } else { "renter" };

    // Check if already submitted
    let existing = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM damage_reports WHERE booking_id = $1 AND reporter_id = $2)",
    )
    .bind(body.booking_id)
    .bind(claims.sub)
    .fetch_one(pool.get_ref())
    .await;

    if let Ok(true) = existing {
        return HttpResponse::Conflict()
            .json(serde_json::json!({"error": "You already submitted a report for this booking"}));
    }

    let id = Uuid::new_v4();
    let result = sqlx::query_as::<_, DamageReport>(
        r#"INSERT INTO damage_reports (id, booking_id, reporter_id, reporter_role, photos, notes,
            odometer_reading, fuel_level, exterior_condition, interior_condition, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
        RETURNING *"#,
    )
    .bind(id)
    .bind(body.booking_id)
    .bind(claims.sub)
    .bind(role)
    .bind(&body.photos)
    .bind(&body.notes)
    .bind(body.odometer_reading)
    .bind(&body.fuel_level)
    .bind(body.exterior_condition.as_deref().unwrap_or("good"))
    .bind(body.interior_condition.as_deref().unwrap_or("good"))
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(report) => HttpResponse::Created().json(report),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

/// GET /api/damage-reports/{booking_id} — Get damage reports for a booking
pub async fn get_reports(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    let booking_id = path.into_inner();

    // Verify user is involved in booking
    let is_involved = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM bookings WHERE id = $1 AND (renter_id = $2 OR host_id = $2))",
    )
    .bind(booking_id)
    .bind(claims.sub)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(false);

    if !is_involved {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": "Not your booking"}));
    }

    let reports = sqlx::query_as::<_, DamageReport>(
        "SELECT * FROM damage_reports WHERE booking_id = $1 ORDER BY created_at",
    )
    .bind(booking_id)
    .fetch_all(pool.get_ref())
    .await;

    match reports {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}
