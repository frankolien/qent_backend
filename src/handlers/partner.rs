use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};

use crate::models::{
    Car, CarStatus, Claims, CreatePartnerApplicationRequest, HostDashboard, PartnerApplication,
    UserRole,
};
use crate::services::AppConfig;

pub async fn apply(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    body: web::Json<CreatePartnerApplicationRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({"errors": e.to_string()}));
    }

    // Check if user already has a pending or approved application
    let existing = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM partner_applications WHERE user_id = $1 AND status IN ('pending', 'approved'))",
    )
    .bind(claims.sub)
    .fetch_one(pool.get_ref())
    .await;

    if let Ok(true) = existing {
        return HttpResponse::Conflict().json(
            serde_json::json!({"error": "You already have an active partnership application"}),
        );
    }

    let app_id = Uuid::new_v4();
    let car_description = body.car_description.clone().unwrap_or_default();
    let fuel_type = body
        .fuel_type
        .clone()
        .unwrap_or_else(|| "petrol".to_string());

    // Create the partner application
    let result = sqlx::query_as::<_, PartnerApplication>(
        r#"INSERT INTO partner_applications
            (id, user_id, full_name, email, phone, drivers_license,
             car_make, car_model, car_year, car_color, car_plate_number,
             car_photos, car_description, fuel_type, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, 'pending', NOW(), NOW())
        RETURNING *"#,
    )
    .bind(app_id)
    .bind(claims.sub)
    .bind(&body.full_name)
    .bind(&body.email)
    .bind(&body.phone)
    .bind(&body.drivers_license)
    .bind(&body.car_make)
    .bind(&body.car_model)
    .bind(body.car_year)
    .bind(&body.car_color)
    .bind(&body.car_plate_number)
    .bind(&body.car_photos)
    .bind(&car_description)
    .bind(&fuel_type)
    .fetch_one(pool.get_ref())
    .await;

    let application = match result {
        Ok(app) => app,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}));
        }
    };

    // Also create a car listing with pendingapproval status
    let car_id = Uuid::new_v4();
    let features: Vec<String> = vec![];

    let _car_result = sqlx::query_as::<_, Car>(
        r#"WITH inserted AS (
            INSERT INTO cars (id, host_id, make, model, year, color, plate_number, description,
                price_per_day, location, latitude, longitude, photos, features, status, seats,
                created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, 5, NOW(), NOW())
            RETURNING *
        )
        SELECT inserted.*, 0.0::double precision as rating, 0::bigint as trip_count,
            u.full_name as host_name
        FROM inserted
        LEFT JOIN users u ON u.id = inserted.host_id"#,
    )
    .bind(car_id)
    .bind(claims.sub)
    .bind(&body.car_make)
    .bind(&body.car_model)
    .bind(body.car_year)
    .bind(&body.car_color)
    .bind(&body.car_plate_number)
    .bind(&car_description)
    .bind(body.price_per_day)
    .bind(&body.location)
    .bind(body.latitude)
    .bind(body.longitude)
    .bind(&body.car_photos)
    .bind(&features)
    .bind(CarStatus::PendingApproval)
    .fetch_one(pool.get_ref())
    .await;

    // Upgrade user role to host
    let _ = sqlx::query("UPDATE users SET role = 'host', updated_at = NOW() WHERE id = $1")
        .bind(claims.sub)
        .execute(pool.get_ref())
        .await;

    // Issue a new JWT with the Host role so the app picks it up immediately
    let new_claims = Claims {
        sub: claims.sub,
        role: UserRole::Host,
        exp: (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };
    let new_token = encode(
        &Header::default(),
        &new_claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .unwrap_or_default();

    HttpResponse::Created().json(serde_json::json!({
        "message": "Partnership application submitted successfully",
        "application": application,
        "token": new_token
    }))
}

/// Activate the partner's most recent car listing (pendingapproval → active)
pub async fn activate_car(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    // Activate all pending cars for this host and approve the application
    let updated = sqlx::query(
        "UPDATE cars SET status = 'active', updated_at = NOW() WHERE host_id = $1 AND status = 'pendingapproval'",
    )
    .bind(claims.sub)
    .execute(pool.get_ref())
    .await;

    // Also approve the partner application
    let _ = sqlx::query(
        "UPDATE partner_applications SET status = 'approved', updated_at = NOW() WHERE user_id = $1 AND status = 'pending'",
    )
    .bind(claims.sub)
    .execute(pool.get_ref())
    .await;

    match updated {
        Ok(result) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Car listing activated successfully",
            "cars_activated": result.rows_affected()
        })),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

pub async fn get_application(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let result = sqlx::query_as::<_, PartnerApplication>(
        "SELECT * FROM partner_applications WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(app)) => HttpResponse::Ok().json(app),
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "No application found"}))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

pub async fn dashboard(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    // Total earnings from completed bookings where user is the host
    let earnings: f64 = sqlx::query_scalar(
        r#"SELECT COALESCE(SUM(p.amount), 0.0)::double precision
        FROM payments p
        JOIN bookings b ON p.booking_id = b.id
        JOIN cars c ON b.car_id = c.id
        WHERE c.host_id = $1 AND b.status = 'completed' AND p.status = 'success'"#,
    )
    .bind(claims.sub)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0.0);

    // Active listings count
    let active_listings: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM cars WHERE host_id = $1 AND status = 'active'",
    )
    .bind(claims.sub)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);

    // Completed bookings count
    let completed_bookings: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*)::bigint
        FROM bookings b
        JOIN cars c ON b.car_id = c.id
        WHERE c.host_id = $1 AND b.status = 'completed'"#,
    )
    .bind(claims.sub)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);

    // Average rating across all host's cars
    let average_rating: f64 = sqlx::query_scalar(
        r#"SELECT COALESCE(AVG(r.rating), 0.0)::double precision
        FROM reviews r
        JOIN bookings b ON r.booking_id = b.id
        JOIN cars c ON b.car_id = c.id
        WHERE c.host_id = $1"#,
    )
    .bind(claims.sub)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0.0);

    HttpResponse::Ok().json(HostDashboard {
        total_earnings: earnings,
        active_listings,
        completed_bookings,
        average_rating,
    })
}
