use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Booking, Car, CarStatus, Claims, Payment, User, UserRole, VerificationStatus};

fn require_admin(req: &HttpRequest) -> Result<Claims, HttpResponse> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})))?;

    if claims.role != UserRole::Admin {
        return Err(HttpResponse::Forbidden().json(serde_json::json!({"error": "Admin access required"})));
    }
    Ok(claims)
}

pub async fn list_users(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let result = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(users) => {
            let public: Vec<crate::models::UserPublic> = users.into_iter().map(Into::into).collect();
            HttpResponse::Ok().json(public)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn list_all_cars(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let result = sqlx::query_as::<_, Car>("SELECT * FROM cars ORDER BY created_at DESC")
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(cars) => HttpResponse::Ok().json(cars),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn approve_car(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let car_id = path.into_inner();
    let result = sqlx::query(
        "UPDATE cars SET status = $1, updated_at = NOW() WHERE id = $2 AND status = 'pendingapproval'",
    )
    .bind(CarStatus::Active)
    .bind(car_id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            HttpResponse::Ok().json(serde_json::json!({"message": "Car approved"}))
        }
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({"error": "Car not found or not pending"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn reject_car(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let car_id = path.into_inner();
    let result = sqlx::query(
        "UPDATE cars SET status = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(CarStatus::Rejected)
    .bind(car_id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            HttpResponse::Ok().json(serde_json::json!({"message": "Car rejected"}))
        }
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({"error": "Car not found"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn verify_user(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let user_id = path.into_inner();
    let result = sqlx::query(
        "UPDATE users SET verification_status = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(VerificationStatus::Verified)
    .bind(user_id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            HttpResponse::Ok().json(serde_json::json!({"message": "User verified"}))
        }
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({"error": "User not found"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn reject_user_verification(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let user_id = path.into_inner();
    let result = sqlx::query(
        "UPDATE users SET verification_status = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(VerificationStatus::Rejected)
    .bind(user_id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            HttpResponse::Ok().json(serde_json::json!({"message": "Verification rejected"}))
        }
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({"error": "User not found"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn deactivate_user(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let user_id = path.into_inner();
    let result = sqlx::query(
        "UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1",
    )
    .bind(user_id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            HttpResponse::Ok().json(serde_json::json!({"message": "User deactivated"}))
        }
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({"error": "User not found"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn get_analytics(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let total_users = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);

    let total_cars = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM cars")
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);

    let total_bookings = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM bookings")
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);

    let total_revenue = sqlx::query_scalar::<_, Option<f64>>(
        "SELECT SUM(amount) FROM payments WHERE status = 'success'",
    )
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(Some(0.0))
    .unwrap_or(0.0);

    let active_bookings = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM bookings WHERE status IN ('confirmed', 'active')",
    )
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);

    let pending_approvals = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM cars WHERE status = 'pendingapproval'",
    )
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);

    HttpResponse::Ok().json(serde_json::json!({
        "total_users": total_users,
        "total_cars": total_cars,
        "total_bookings": total_bookings,
        "total_revenue": total_revenue,
        "active_bookings": active_bookings,
        "pending_car_approvals": pending_approvals,
    }))
}

pub async fn list_all_bookings(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let result = sqlx::query_as::<_, Booking>(
        "SELECT * FROM bookings ORDER BY created_at DESC LIMIT 100",
    )
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(bookings) => HttpResponse::Ok().json(bookings),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn list_all_payments(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let result = sqlx::query_as::<_, Payment>(
        "SELECT * FROM payments ORDER BY created_at DESC LIMIT 100",
    )
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(payments) => HttpResponse::Ok().json(payments),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn handle_dispute_refund(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let booking_id = path.into_inner();

    let booking = match sqlx::query_as::<_, Booking>("SELECT * FROM bookings WHERE id = $1")
        .bind(booking_id)
        .fetch_optional(pool.get_ref())
        .await
    {
        Ok(Some(b)) => b,
        Ok(None) => return HttpResponse::NotFound().json(serde_json::json!({"error": "Booking not found"})),
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    // Full refund for admin disputes
    let _ = sqlx::query(
        r#"INSERT INTO payments (id, booking_id, payer_id, amount, currency, provider, status, transaction_type, created_at)
        VALUES ($1, $2, $3, $4, 'NGN', 'paystack', 'success', 'refund', NOW())"#,
    )
    .bind(Uuid::new_v4())
    .bind(booking_id)
    .bind(booking.renter_id)
    .bind(booking.total_amount)
    .execute(pool.get_ref())
    .await;

    let _ = sqlx::query(
        "UPDATE bookings SET status = 'cancelled', cancellation_reason = 'Admin dispute resolution', updated_at = NOW() WHERE id = $1",
    )
    .bind(booking_id)
    .execute(pool.get_ref())
    .await;

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Dispute resolved, full refund issued",
        "refund_amount": booking.total_amount
    }))
}
