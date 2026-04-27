use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    Booking, BookingAction, BookingActionRequest, BookingStatus, BookingWithCar, Car, Claims,
    CreateBookingRequest, ProtectionPlan, UserRole,
};
use crate::services::email::EmailService;
use crate::services::push::PushService;
use crate::services::AppConfig;

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
            // Notify host about new booking request
            let _ = create_notification(
                pool.get_ref(),
                push.get_ref().as_ref(),
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

    let booking = match sqlx::query_as::<_, Booking>("SELECT * FROM bookings WHERE id = $1")
        .bind(booking_id)
        .fetch_optional(pool.get_ref())
        .await
    {
        Ok(Some(b)) => b,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({"error": "Booking not found"}))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };

    let new_status = match body.action {
        BookingAction::Approve => {
            if booking.host_id != claims.sub && claims.role != UserRole::Admin {
                return HttpResponse::Forbidden()
                    .json(serde_json::json!({"error": "Only the host can approve"}));
            }
            if booking.status != BookingStatus::Pending {
                return HttpResponse::BadRequest()
                    .json(serde_json::json!({"error": "Booking is not pending"}));
            }
            BookingStatus::Approved
        }
        BookingAction::Reject => {
            if booking.host_id != claims.sub && claims.role != UserRole::Admin {
                return HttpResponse::Forbidden()
                    .json(serde_json::json!({"error": "Only the host can reject"}));
            }
            BookingStatus::Rejected
        }
        BookingAction::Cancel => {
            if booking.renter_id != claims.sub
                && booking.host_id != claims.sub
                && claims.role != UserRole::Admin
            {
                return HttpResponse::Forbidden()
                    .json(serde_json::json!({"error": "Not authorized to cancel"}));
            }
            BookingStatus::Cancelled
        }
        BookingAction::Activate => {
            if booking.host_id != claims.sub && claims.role != UserRole::Admin {
                return HttpResponse::Forbidden()
                    .json(serde_json::json!({"error": "Only the host can activate"}));
            }
            if booking.status != BookingStatus::Approved
                && booking.status != BookingStatus::Confirmed
            {
                return HttpResponse::BadRequest().json(serde_json::json!({"error": "Booking must be approved or confirmed to activate"}));
            }
            BookingStatus::Active
        }
        BookingAction::Complete => {
            if booking.host_id != claims.sub && claims.role != UserRole::Admin {
                return HttpResponse::Forbidden()
                    .json(serde_json::json!({"error": "Only the host can complete"}));
            }
            if booking.status != BookingStatus::Active {
                return HttpResponse::BadRequest()
                    .json(serde_json::json!({"error": "Booking is not active"}));
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

    // Fetch car name for notification
    let car_name =
        sqlx::query_scalar::<_, String>("SELECT make || ' ' || model FROM cars WHERE id = $1")
            .bind(booking.car_id)
            .fetch_optional(pool.get_ref())
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "your car".to_string());

    match &result {
        Ok(b) => {
            let data = Some(serde_json::json!({"booking_id": b.id.to_string()}));
            match new_status {
                BookingStatus::Approved => {
                    let _ = create_notification(
                        pool.get_ref(),
                        push.get_ref().as_ref(),
                        booking.renter_id,
                        "Booking Approved",
                        &format!("Your booking for {} has been approved! Coordinate pickup with the host.", car_name),
                        "booking_approved", data,
                    ).await;
                }
                BookingStatus::Rejected => {
                    let _ = create_notification(
                        pool.get_ref(),
                        push.get_ref().as_ref(),
                        booking.renter_id,
                        "Booking Declined",
                        &format!("Your booking for {} was declined by the host.", car_name),
                        "booking_rejected",
                        data,
                    )
                    .await;
                }
                BookingStatus::Cancelled => {
                    // Notify the other party
                    let notify_user = if claims.sub == booking.renter_id {
                        booking.host_id
                    } else {
                        booking.renter_id
                    };
                    let _ = create_notification(
                        pool.get_ref(),
                        push.get_ref().as_ref(),
                        notify_user,
                        "Booking Cancelled",
                        &format!("A booking for {} has been cancelled.", car_name),
                        "booking_cancelled",
                        data,
                    )
                    .await;
                }
                BookingStatus::Active => {
                    let _ = create_notification(
                        pool.get_ref(),
                        push.get_ref().as_ref(),
                        booking.renter_id,
                        "Trip Started",
                        &format!(
                            "Your trip with {} is now active. Enjoy your ride!",
                            car_name
                        ),
                        "booking_active",
                        data,
                    )
                    .await;
                }
                BookingStatus::Completed => {
                    let _ = create_notification(
                        pool.get_ref(),
                        push.get_ref().as_ref(),
                        booking.renter_id,
                        "Trip Completed",
                        &format!("Your trip with {} is complete. Leave a review!", car_name),
                        "booking_completed",
                        data,
                    )
                    .await;
                }
                _ => {}
            }

            // Send status change email
            let email_service = EmailService::new(config.resend_api_key.clone());
            let (notify_user_id, email_msg) = match new_status {
                BookingStatus::Approved => (
                    booking.renter_id,
                    format!(
                        "Your booking for {} has been approved! Coordinate pickup with the host.",
                        car_name
                    ),
                ),
                BookingStatus::Rejected => (
                    booking.renter_id,
                    format!("Your booking for {} was declined by the host.", car_name),
                ),
                BookingStatus::Cancelled => {
                    let other = if claims.sub == booking.renter_id {
                        booking.host_id
                    } else {
                        booking.renter_id
                    };
                    (
                        other,
                        format!("A booking for {} has been cancelled.", car_name),
                    )
                }
                BookingStatus::Active => (
                    booking.renter_id,
                    format!(
                        "Your trip with {} is now active. Enjoy your ride!",
                        car_name
                    ),
                ),
                BookingStatus::Completed => (
                    booking.renter_id,
                    format!(
                        "Your trip with {} is complete. We'd love your feedback!",
                        car_name
                    ),
                ),
                _ => (booking.renter_id, String::new()),
            };

            if !email_msg.is_empty() {
                let user_info = sqlx::query_as::<_, (String, String)>(
                    "SELECT email, full_name FROM users WHERE id = $1",
                )
                .bind(notify_user_id)
                .fetch_optional(pool.get_ref())
                .await;

                if let Ok(Some((email, name))) = user_info {
                    let status_str = format!("{:?}", new_status).to_lowercase();
                    email_service
                        .send_status_email(&email, &name, &car_name, &status_str, &email_msg)
                        .await;
                }
            }
        }
        Err(_) => {}
    }

    match result {
        Ok(b) => HttpResponse::Ok().json(b),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// Helper to create a notification record
async fn create_notification(
    pool: &PgPool,
    push: Option<&PushService>,
    user_id: Uuid,
    title: &str,
    message: &str,
    notification_type: &str,
    data: Option<serde_json::Value>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO notifications (id, user_id, title, message, notification_type, is_read, data, created_at)
        VALUES ($1, $2, $3, $4, $5, false, $6, NOW())"#,
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(title)
    .bind(message)
    .bind(notification_type)
    .bind(data.clone())
    .execute(pool)
    .await?;

    if let Some(push) = push {
        let payload = data.unwrap_or_else(|| serde_json::json!({}));
        let pool = pool.clone();
        let push = push.clone();
        let title = title.to_string();
        let message = message.to_string();
        tokio::spawn(async move {
            push.send_to_user(&pool, user_id, &title, &message, payload).await;
        });
    }

    Ok(())
}

/// Get pending bookings for host (bookings awaiting their approval)
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
