use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    Booking, BookingAction, BookingActionRequest, BookingStatus, Car, Claims,
    CreateBookingRequest, ProtectionPlan, UserRole,
};

pub async fn create_booking(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<CreateBookingRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    // Fetch car
    let car = match sqlx::query_as::<_, Car>("SELECT * FROM cars WHERE id = $1 AND status = 'active'")
        .bind(body.car_id)
        .fetch_optional(pool.get_ref())
        .await
    {
        Ok(Some(c)) => c,
        Ok(None) => return HttpResponse::NotFound().json(serde_json::json!({"error": "Car not found or not available"})),
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    if car.host_id == claims.sub {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Cannot book your own car"}));
    }

    // Check date overlap
    let overlap = sqlx::query_scalar::<_, bool>(
        r#"SELECT EXISTS(
            SELECT 1 FROM bookings
            WHERE car_id = $1
            AND status IN ('pending', 'approved', 'confirmed', 'active')
            AND start_date <= $3 AND end_date >= $2
        )"#,
    )
    .bind(body.car_id)
    .bind(body.start_date)
    .bind(body.end_date)
    .fetch_one(pool.get_ref())
    .await;

    if let Ok(true) = overlap {
        return HttpResponse::Conflict().json(serde_json::json!({"error": "Car is already booked for these dates"}));
    }

    let total_days = (body.end_date - body.start_date).num_days() as i32;
    if total_days <= 0 {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "End date must be after start date"}));
    }

    let subtotal = car.price_per_day * total_days as f64;
    let service_fee = subtotal * 0.10; // 10% service fee

    // Protection plan fee
    let mut protection_fee = 0.0;
    if let Some(plan_id) = body.protection_plan_id {
        if let Ok(Some(plan)) = sqlx::query_as::<_, ProtectionPlan>(
            "SELECT * FROM protection_plans WHERE id = $1 AND is_active = true",
        )
        .bind(plan_id)
        .fetch_optional(pool.get_ref())
        .await
        {
            protection_fee = plan.daily_rate * total_days as f64;
        }
    }

    let total_amount = subtotal + service_fee + protection_fee;
    let id = Uuid::new_v4();

    let result = sqlx::query_as::<_, Booking>(
        r#"INSERT INTO bookings (id, car_id, renter_id, host_id, start_date, end_date, total_days,
            price_per_day, subtotal, protection_plan_id, protection_fee, service_fee, total_amount,
            status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, NOW(), NOW())
        RETURNING *"#,
    )
    .bind(id)
    .bind(body.car_id)
    .bind(claims.sub)
    .bind(car.host_id)
    .bind(body.start_date)
    .bind(body.end_date)
    .bind(total_days)
    .bind(car.price_per_day)
    .bind(subtotal)
    .bind(body.protection_plan_id)
    .bind(protection_fee)
    .bind(service_fee)
    .bind(total_amount)
    .bind(BookingStatus::Pending)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(booking) => HttpResponse::Created().json(booking),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn get_booking(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    let booking_id = path.into_inner();

    let result = sqlx::query_as::<_, Booking>(
        "SELECT * FROM bookings WHERE id = $1 AND (renter_id = $2 OR host_id = $2 OR $3)",
    )
    .bind(booking_id)
    .bind(claims.sub)
    .bind(claims.role == UserRole::Admin)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(b)) => HttpResponse::Ok().json(b),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({"error": "Booking not found"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn get_my_bookings(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    let result = sqlx::query_as::<_, Booking>(
        "SELECT * FROM bookings WHERE renter_id = $1 OR host_id = $1 ORDER BY created_at DESC",
    )
    .bind(claims.sub)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(bookings) => HttpResponse::Ok().json(bookings),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn update_booking_status(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<BookingActionRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

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

    let new_status = match body.action {
        BookingAction::Approve => {
            if booking.host_id != claims.sub && claims.role != UserRole::Admin {
                return HttpResponse::Forbidden().json(serde_json::json!({"error": "Only the host can approve"}));
            }
            if booking.status != BookingStatus::Pending {
                return HttpResponse::BadRequest().json(serde_json::json!({"error": "Booking is not pending"}));
            }
            BookingStatus::Approved
        }
        BookingAction::Reject => {
            if booking.host_id != claims.sub && claims.role != UserRole::Admin {
                return HttpResponse::Forbidden().json(serde_json::json!({"error": "Only the host can reject"}));
            }
            BookingStatus::Rejected
        }
        BookingAction::Cancel => {
            if booking.renter_id != claims.sub && booking.host_id != claims.sub && claims.role != UserRole::Admin {
                return HttpResponse::Forbidden().json(serde_json::json!({"error": "Not authorized to cancel"}));
            }
            BookingStatus::Cancelled
        }
        BookingAction::Complete => {
            if booking.host_id != claims.sub && claims.role != UserRole::Admin {
                return HttpResponse::Forbidden().json(serde_json::json!({"error": "Only the host can complete"}));
            }
            if booking.status != BookingStatus::Active {
                return HttpResponse::BadRequest().json(serde_json::json!({"error": "Booking is not active"}));
            }
            BookingStatus::Completed
        }
    };

    let result = sqlx::query_as::<_, Booking>(
        r#"UPDATE bookings SET status = $1, cancellation_reason = $2, updated_at = NOW()
        WHERE id = $3 RETURNING *"#,
    )
    .bind(&new_status)
    .bind(&body.reason)
    .bind(booking_id)
    .fetch_one(pool.get_ref())
    .await;

    // If completed, credit host wallet
    if new_status == BookingStatus::Completed {
        let host_payout = booking.subtotal * 0.85; // Host gets 85% (platform takes 15%)
        let _ = sqlx::query(
            "UPDATE users SET wallet_balance = wallet_balance + $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(host_payout)
        .bind(booking.host_id)
        .execute(pool.get_ref())
        .await;

        let _ = sqlx::query(
            r#"INSERT INTO wallet_transactions (id, user_id, amount, balance_after, description, reference_id, created_at)
            VALUES ($1, $2, $3, (SELECT wallet_balance FROM users WHERE id = $2), $4, $5, NOW())"#,
        )
        .bind(Uuid::new_v4())
        .bind(booking.host_id)
        .bind(host_payout)
        .bind(format!("Payout for booking {}", booking_id))
        .bind(booking_id)
        .execute(pool.get_ref())
        .await;
    }

    match result {
        Ok(b) => HttpResponse::Ok().json(b),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}
