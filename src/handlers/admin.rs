use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    Booking, Car, CarStatus, Claims, Payment, User, UserRole, VerificationStatus, WalletTransaction,
};
use crate::services::AppConfig;

fn require_admin(req: &HttpRequest) -> Result<Claims, HttpResponse> {
    let claims = req.extensions().get::<Claims>().cloned().ok_or_else(|| {
        HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
    })?;

    if claims.role != UserRole::Admin {
        return Err(
            HttpResponse::Forbidden().json(serde_json::json!({"error": "Admin access required"}))
        );
    }
    Ok(claims)
}

/// GET /api/admin/users — List all users (admin only)
#[utoipa::path(
    get,
    path = "/api/admin/users",
    tag = "Admin",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "All users (public profile shape)", body = Vec<crate::models::UserPublic>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
    ),
)]
pub async fn list_users(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let result = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(users) => {
            let public: Vec<crate::models::UserPublic> =
                users.into_iter().map(Into::into).collect();
            HttpResponse::Ok().json(public)
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// GET /api/admin/cars — List all cars across all hosts (admin only)
#[utoipa::path(
    get,
    path = "/api/admin/cars",
    tag = "Admin",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "All cars with rating + host info", body = Vec<Car>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
    ),
)]
pub async fn list_all_cars(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let result = sqlx::query_as::<_, Car>(
        r#"SELECT c.*,
            COALESCE(rs.avg_rating, 0.0) as rating,
            COALESCE(rs.trip_count, 0) as trip_count,
            u.full_name as host_name
        FROM cars c
        LEFT JOIN (
            SELECT b.car_id,
                   AVG(r.rating)::double precision as avg_rating,
                   COUNT(DISTINCT b.id) as trip_count
            FROM reviews r
            JOIN bookings b ON r.booking_id = b.id
            GROUP BY b.car_id
        ) rs ON rs.car_id = c.id
        LEFT JOIN users u ON u.id = c.host_id
        ORDER BY c.created_at DESC"#,
    )
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(cars) => HttpResponse::Ok().json(cars),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// POST /api/admin/cars/{id}/approve — Approve a car listing (pending → active)
#[utoipa::path(
    post,
    path = "/api/admin/cars/{id}/approve",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Car ID")),
    responses(
        (status = 200, description = "Car approved"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "Car not found or not pending"),
    ),
)]
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
        Ok(_) => HttpResponse::NotFound()
            .json(serde_json::json!({"error": "Car not found or not pending"})),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// POST /api/admin/cars/{id}/reject — Reject a car listing
#[utoipa::path(
    post,
    path = "/api/admin/cars/{id}/reject",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Car ID")),
    responses(
        (status = 200, description = "Car rejected"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "Car not found"),
    ),
)]
pub async fn reject_car(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let car_id = path.into_inner();
    let result = sqlx::query("UPDATE cars SET status = $1, updated_at = NOW() WHERE id = $2")
        .bind(CarStatus::Rejected)
        .bind(car_id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            HttpResponse::Ok().json(serde_json::json!({"message": "Car rejected"}))
        }
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({"error": "Car not found"})),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// POST /api/admin/users/{id}/verify — Mark user identity as verified
#[utoipa::path(
    post,
    path = "/api/admin/users/{id}/verify",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "User verified"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "User not found"),
    ),
)]
pub async fn verify_user(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let user_id = path.into_inner();
    let result =
        sqlx::query("UPDATE users SET verification_status = $1, updated_at = NOW() WHERE id = $2")
            .bind(VerificationStatus::Verified)
            .bind(user_id)
            .execute(pool.get_ref())
            .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            HttpResponse::Ok().json(serde_json::json!({"message": "User verified"}))
        }
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({"error": "User not found"})),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// POST /api/admin/users/{id}/reject — Reject user identity verification
#[utoipa::path(
    post,
    path = "/api/admin/users/{id}/reject",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "Verification rejected"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "User not found"),
    ),
)]
pub async fn reject_user_verification(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let user_id = path.into_inner();
    let result =
        sqlx::query("UPDATE users SET verification_status = $1, updated_at = NOW() WHERE id = $2")
            .bind(VerificationStatus::Rejected)
            .bind(user_id)
            .execute(pool.get_ref())
            .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            HttpResponse::Ok().json(serde_json::json!({"message": "Verification rejected"}))
        }
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({"error": "User not found"})),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// POST /api/admin/users/{id}/deactivate — Disable a user account
#[utoipa::path(
    post,
    path = "/api/admin/users/{id}/deactivate",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "User deactivated"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "User not found"),
    ),
)]
pub async fn deactivate_user(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let user_id = path.into_inner();
    let result =
        sqlx::query("UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(user_id)
            .execute(pool.get_ref())
            .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            HttpResponse::Ok().json(serde_json::json!({"message": "User deactivated"}))
        }
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({"error": "User not found"})),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// GET /api/admin/analytics — Platform-wide stats (users, cars, bookings, revenue)
#[utoipa::path(
    get,
    path = "/api/admin/analytics",
    tag = "Admin",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "{ total_users, total_cars, total_bookings, total_revenue, active_bookings, pending_car_approvals }"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
    ),
)]
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

    let pending_approvals =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM cars WHERE status = 'pendingapproval'")
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

/// GET /api/admin/bookings — Latest 100 bookings across the platform
#[utoipa::path(
    get,
    path = "/api/admin/bookings",
    tag = "Admin",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Bookings", body = Vec<Booking>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
    ),
)]
pub async fn list_all_bookings(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let result =
        sqlx::query_as::<_, Booking>("SELECT * FROM bookings ORDER BY created_at DESC LIMIT 100")
            .fetch_all(pool.get_ref())
            .await;

    match result {
        Ok(bookings) => HttpResponse::Ok().json(bookings),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// GET /api/admin/payments — Latest 100 payments across the platform
#[utoipa::path(
    get,
    path = "/api/admin/payments",
    tag = "Admin",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Payments", body = Vec<Payment>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
    ),
)]
pub async fn list_all_payments(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let result =
        sqlx::query_as::<_, Payment>("SELECT * FROM payments ORDER BY created_at DESC LIMIT 100")
            .fetch_all(pool.get_ref())
            .await;

    match result {
        Ok(payments) => HttpResponse::Ok().json(payments),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// POST /api/admin/bookings/{id}/dispute-refund — Cancel booking + issue full refund
#[utoipa::path(
    post,
    path = "/api/admin/bookings/{id}/dispute-refund",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Booking ID")),
    responses(
        (status = 200, description = "Refund issued + booking cancelled"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "Booking not found"),
    ),
)]
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
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({"error": "Booking not found"}))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
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

/// GET /api/admin/withdrawals/pending — List pending withdrawal approvals
#[utoipa::path(
    get,
    path = "/api/admin/withdrawals/pending",
    tag = "Admin",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Pending withdrawal transactions", body = Vec<WalletTransaction>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
    ),
)]
pub async fn list_pending_withdrawals(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let result = sqlx::query_as::<_, WalletTransaction>(
        r#"SELECT wt.* FROM wallet_transactions wt
        WHERE wt.status = 'pending_approval'
        ORDER BY wt.created_at DESC"#,
    )
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(txns) => HttpResponse::Ok().json(txns),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// POST /api/admin/withdrawals/{id}/approve — Approve a pending withdrawal
#[utoipa::path(
    post,
    path = "/api/admin/withdrawals/{id}/approve",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Wallet transaction ID")),
    responses(
        (status = 200, description = "Withdrawal approved"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "Pending withdrawal not found"),
    ),
)]
pub async fn approve_withdrawal(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let txn_id = path.into_inner();

    // Get the pending transaction
    let txn = sqlx::query_as::<_, WalletTransaction>(
        "SELECT * FROM wallet_transactions WHERE id = $1 AND status = 'pending_approval'",
    )
    .bind(txn_id)
    .fetch_optional(pool.get_ref())
    .await;

    let txn = match txn {
        Ok(Some(t)) => t,
        _ => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": "Pending withdrawal not found"}))
        }
    };

    // Mark as completed
    let _ = sqlx::query("UPDATE wallet_transactions SET status = 'completed' WHERE id = $1")
        .bind(txn_id)
        .execute(pool.get_ref())
        .await;

    // Notify user
    let _ = sqlx::query(
        r#"INSERT INTO notifications (id, user_id, title, message, notification_type, is_read, created_at)
        VALUES ($1, $2, 'Withdrawal Approved', 'Your withdrawal has been approved and is being processed.', 'withdrawal_approved', false, NOW())"#,
    )
    .bind(Uuid::new_v4())
    .bind(txn.user_id)
    .execute(pool.get_ref())
    .await;

    let _ = config; // future: trigger actual Paystack transfer here

    HttpResponse::Ok()
        .json(serde_json::json!({"message": "Withdrawal approved", "transaction_id": txn_id}))
}

/// POST /api/admin/withdrawals/{id}/reject — Reject and refund a pending withdrawal
#[utoipa::path(
    post,
    path = "/api/admin/withdrawals/{id}/reject",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Wallet transaction ID")),
    responses(
        (status = 200, description = "Withdrawal rejected, funds returned to wallet"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "Pending withdrawal not found"),
    ),
)]
pub async fn reject_withdrawal(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let txn_id = path.into_inner();

    let txn = sqlx::query_as::<_, WalletTransaction>(
        "SELECT * FROM wallet_transactions WHERE id = $1 AND status = 'pending_approval'",
    )
    .bind(txn_id)
    .fetch_optional(pool.get_ref())
    .await;

    let txn = match txn {
        Ok(Some(t)) => t,
        _ => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": "Pending withdrawal not found"}))
        }
    };

    // Refund the held amount back to wallet
    let refund_amount = txn.amount.abs();
    let _ = sqlx::query(
        "UPDATE users SET wallet_balance = wallet_balance + $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(refund_amount)
    .bind(txn.user_id)
    .execute(pool.get_ref())
    .await;

    // Mark as rejected
    let _ = sqlx::query("UPDATE wallet_transactions SET status = 'rejected', admin_notes = 'Rejected by admin' WHERE id = $1")
        .bind(txn_id)
        .execute(pool.get_ref())
        .await;

    // Notify user
    let _ = sqlx::query(
        r#"INSERT INTO notifications (id, user_id, title, message, notification_type, is_read, created_at)
        VALUES ($1, $2, 'Withdrawal Rejected', 'Your withdrawal was rejected and the funds have been returned to your wallet.', 'withdrawal_rejected', false, NOW())"#,
    )
    .bind(Uuid::new_v4())
    .bind(txn.user_id)
    .execute(pool.get_ref())
    .await;

    HttpResponse::Ok().json(serde_json::json!({"message": "Withdrawal rejected, funds returned", "transaction_id": txn_id}))
}
