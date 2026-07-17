use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    Booking, BookingActionRequest, BookingStatus, BookingWithCar, Car, Claims,
    CreateBookingRequest, ProtectionPlan, UserRole,
};
use crate::services::booking_notifications::BookingNotifications;
use crate::services::booking_workflows::{BookingWorkflowError, BookingWorkflows};
use crate::services::email::EmailService;
use crate::services::push::PushService;
use crate::services::AppConfig;

/// POST /api/bookings — Renter creates a new booking request for a car
#[utoipa::path(
    post,
    path = "/api/bookings",
    tag = "Bookings",
    security(("bearer_auth" = [])),
    request_body = CreateBookingRequest,
    responses(
        (status = 201, description = "Booking created in pending state", body = Booking),
        (status = 400, description = "Invalid dates or self-booking attempt"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Car not found or not available"),
        (status = 409, description = "Car already booked for these dates"),
    ),
)]
pub async fn create_booking(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    push: web::Data<Option<PushService>>,
    body: web::Json<CreateBookingRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    // Fetch car
    let car = match sqlx::query_as::<_, Car>(
        r#"SELECT c.*,
            COALESCE(rs.avg_rating, 0.0) as rating,
            COALESCE(rs.trip_count, 0) as trip_count,
            u.full_name as host_name
        FROM cars c
        LEFT JOIN users u ON u.id = c.host_id
        LEFT JOIN (
            SELECT b.car_id,
                   AVG(r.rating)::double precision as avg_rating,
                   COUNT(DISTINCT b.id) as trip_count
            FROM bookings b
            LEFT JOIN reviews r ON r.booking_id = b.id
            GROUP BY b.car_id
        ) rs ON rs.car_id = c.id
        WHERE c.id = $1 AND c.status = 'active'"#,
    )
    .bind(body.car_id)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": "Car not found or not available"}))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };

    if car.host_id == claims.sub {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Cannot book your own car"}));
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
        return HttpResponse::Conflict()
            .json(serde_json::json!({"error": "Car is already booked for these dates"}));
    }

    let total_days = (body.end_date - body.start_date).num_days() as i32;
    if total_days <= 0 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "End date must be after start date"}));
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
        Ok(booking) => {
            let email_service = EmailService::new(String::new());
            let notifications = BookingNotifications {
                pool: pool.get_ref(),
                push: push.get_ref().as_ref(),
                email: &email_service,
            };
            let _ = notifications
                .create_notification(
                    car.host_id,
                    "New Booking Request",
                    &format!(
                        "You have a new booking request for your {} {}",
                        car.make, car.model
                    ),
                    "booking_request",
                    Some(serde_json::json!({"booking_id": booking.id.to_string()})),
                )
                .await;
            HttpResponse::Created().json(booking)
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// GET /api/bookings/{id} — Fetch a single booking (renter, host, or admin only)
#[utoipa::path(
    get,
    path = "/api/bookings/{id}",
    tag = "Bookings",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Booking ID")),
    responses(
        (status = 200, description = "Booking details", body = Booking),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Booking not found"),
    ),
)]
pub async fn get_booking(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
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
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "Booking not found"}))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// GET /api/bookings/mine — All bookings where the user is renter or host
#[utoipa::path(
    get,
    path = "/api/bookings/mine",
    tag = "Bookings",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of bookings (with car details)", body = Vec<BookingWithCar>),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn get_my_bookings(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let result = sqlx::query_as::<_, BookingWithCar>(
        r#"SELECT b.*,
            (c.make || ' ' || c.model || ' ' || c.year::text) as car_name,
            c.photos[1] as car_photo,
            c.location as car_location,
            u.full_name as renter_name
        FROM bookings b
        LEFT JOIN cars c ON c.id = b.car_id
        LEFT JOIN users u ON u.id = b.renter_id
        WHERE b.renter_id = $1 OR b.host_id = $1
        ORDER BY b.created_at DESC"#,
    )
    .bind(claims.sub)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(bookings) => HttpResponse::Ok().json(bookings),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// POST /api/bookings/{id}/action — Approve, reject, cancel, activate, or complete a booking
#[utoipa::path(
    post,
    path = "/api/bookings/{id}/action",
    tag = "Bookings",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Booking ID")),
    request_body = BookingActionRequest,
    responses(
        (status = 200, description = "Updated booking", body = Booking),
        (status = 400, description = "Invalid state transition"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not allowed for this user/role"),
        (status = 404, description = "Booking not found"),
    ),
)]
pub async fn update_booking_status(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    push: web::Data<Option<PushService>>,
    path: web::Path<Uuid>,
    body: web::Json<BookingActionRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let booking_id = path.into_inner();

    let transition =
        match BookingWorkflows::apply_action(pool.get_ref(), booking_id, &claims, &body).await {
            Ok(t) => t,
            Err(BookingWorkflowError::BookingNotFound) => {
                return HttpResponse::NotFound()
                    .json(serde_json::json!({"error": "Booking not found"}))
            }
            Err(BookingWorkflowError::Forbidden(msg)) => {
                return HttpResponse::Forbidden().json(serde_json::json!({"error": msg}))
            }
            Err(BookingWorkflowError::InvalidTransition(msg)) => {
                return HttpResponse::BadRequest().json(serde_json::json!({"error": msg}))
            }
            Err(e) => {
                log::error!("booking workflow error for {}: {}", booking_id, e);
                return HttpResponse::InternalServerError()
                    .json(serde_json::json!({"error": "Failed to update booking"}));
            }
        };

    let booking = transition.before;
    let updated_booking = transition.after;

    // Fetch car name for notification
    let car_name =
        sqlx::query_scalar::<_, String>("SELECT make || ' ' || model FROM cars WHERE id = $1")
            .bind(booking.car_id)
            .fetch_optional(pool.get_ref())
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "your car".to_string());

    let email_service = EmailService::new(config.resend_api_key.clone());
    let notifications = BookingNotifications {
        pool: pool.get_ref(),
        push: push.get_ref().as_ref(),
        email: &email_service,
    };
    notifications
        .send_status_changed(&claims, &booking, &updated_booking, &car_name)
        .await;

    HttpResponse::Ok().json(updated_booking)
}

/// GET /api/bookings/host/pending — Bookings awaiting host approval
#[utoipa::path(
    get,
    path = "/api/bookings/host/pending",
    tag = "Bookings",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Pending bookings", body = Vec<BookingWithCar>),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn get_host_pending_bookings(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let result = sqlx::query_as::<_, BookingWithCar>(
        r#"SELECT b.*,
            (c.make || ' ' || c.model || ' ' || c.year::text) as car_name,
            c.photos[1] as car_photo,
            c.location as car_location,
            u.full_name as renter_name
        FROM bookings b
        LEFT JOIN cars c ON c.id = b.car_id
        LEFT JOIN users u ON u.id = b.renter_id
        WHERE b.host_id = $1 AND b.status = 'pending'
        ORDER BY b.created_at DESC"#,
    )
    .bind(claims.sub)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(bookings) => HttpResponse::Ok().json(bookings),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}
