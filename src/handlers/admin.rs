use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    Booking, Car, CarStatus, Claims, Payment, User, UserRole, VerificationStatus, WalletTransaction,
};
use crate::services::AppConfig;

pub(crate) fn require_admin(req: &HttpRequest) -> Result<Claims, HttpResponse> {
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

// ─── Partner v2 listings (admin review) ────────────────────────────────────

/// GET /api/admin/partner-listings — list submissions awaiting review.
/// Optional `?status=submitted|in_review|approved|rejected`; default returns
/// only submissions in the review queue (submitted + in_review).
#[utoipa::path(
    get,
    path = "/api/admin/partner-listings",
    tag = "Admin",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Partner listings with host + profile info"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
    ),
)]
pub async fn list_partner_listings(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let status_filter = query.get("status").cloned();

    let sql = r#"
        SELECT
            l.id, l.application_ref, l.user_id, l.tier, l.brand, l.model, l.year,
            l.color, l.plate_number, l.photos, l.listing_status, l.rejection_reason,
            l.vehicle_registration_url, l.insurance_certificate_url,
            l.insurance_policy_number, l.vehicle_plate_frsc_verified,
            l.insurance_niid_verified, l.owner_consent_required,
            l.created_at, l.updated_at,
            p.legal_full_name, p.contract_email, p.phone, p.identity_status,
            p.drivers_license_number, p.drivers_license_front_url,
            p.drivers_license_back_url, p.profile_photo_url,
            u.full_name AS user_full_name, u.email AS user_email,
            u.kyc_tier AS user_kyc_tier
        FROM partner_listings l
        JOIN partner_profiles p ON p.id = l.profile_id
        JOIN users u ON u.id = l.user_id
        WHERE ($1::text IS NULL AND l.listing_status IN ('submitted','in_review'))
           OR ($1::text IS NOT NULL AND l.listing_status = $1::text)
        ORDER BY l.created_at DESC
    "#;

    let rows = sqlx::query(sql)
        .bind(status_filter)
        .fetch_all(pool.get_ref())
        .await;

    match rows {
        Ok(records) => {
            use sqlx::Row;
            let payload: Vec<serde_json::Value> = records
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.get::<Uuid, _>("id"),
                        "application_ref": r.get::<String, _>("application_ref"),
                        "user_id": r.get::<Uuid, _>("user_id"),
                        "tier": r.get::<String, _>("tier"),
                        "brand": r.get::<String, _>("brand"),
                        "model": r.get::<String, _>("model"),
                        "year": r.try_get::<Option<i32>, _>("year").ok().flatten(),
                        "color": r.try_get::<Option<String>, _>("color").ok().flatten(),
                        "plate_number": r.get::<String, _>("plate_number"),
                        "photos": r.get::<Vec<String>, _>("photos"),
                        "listing_status": r.get::<String, _>("listing_status"),
                        "rejection_reason": r.try_get::<Option<String>, _>("rejection_reason").ok().flatten(),
                        "vehicle_registration_url": r.try_get::<Option<String>, _>("vehicle_registration_url").ok().flatten(),
                        "insurance_certificate_url": r.try_get::<Option<String>, _>("insurance_certificate_url").ok().flatten(),
                        "insurance_policy_number": r.try_get::<Option<String>, _>("insurance_policy_number").ok().flatten(),
                        "vehicle_plate_frsc_verified": r.get::<bool, _>("vehicle_plate_frsc_verified"),
                        "insurance_niid_verified": r.get::<bool, _>("insurance_niid_verified"),
                        "owner_consent_required": r.get::<bool, _>("owner_consent_required"),
                        "created_at": r.get::<chrono::NaiveDateTime, _>("created_at"),
                        "updated_at": r.get::<chrono::NaiveDateTime, _>("updated_at"),
                        "profile": {
                            "legal_full_name": r.get::<String, _>("legal_full_name"),
                            "contract_email": r.get::<String, _>("contract_email"),
                            "phone": r.get::<String, _>("phone"),
                            "identity_status": r.get::<String, _>("identity_status"),
                            "drivers_license_number": r.get::<String, _>("drivers_license_number"),
                            "drivers_license_front_url": r.try_get::<Option<String>, _>("drivers_license_front_url").ok().flatten(),
                            "drivers_license_back_url": r.try_get::<Option<String>, _>("drivers_license_back_url").ok().flatten(),
                            "profile_photo_url": r.try_get::<Option<String>, _>("profile_photo_url").ok().flatten(),
                        },
                        "user": {
                            "full_name": r.get::<String, _>("user_full_name"),
                            "email": r.get::<String, _>("user_email"),
                            "kyc_tier": r.get::<i32, _>("user_kyc_tier"),
                        },
                    })
                })
                .collect();
            HttpResponse::Ok().json(payload)
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()})),
    }
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct AdminRejectBody {
    pub reason: Option<String>,
}

/// POST /api/admin/partner-listings/{id}/approve
#[utoipa::path(
    post,
    path = "/api/admin/partner-listings/{id}/approve",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Partner listing ID")),
    responses(
        (status = 200, description = "Listing approved"),
        (status = 404, description = "Listing not found or not in a reviewable state"),
    ),
)]
pub async fn approve_partner_listing(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let listing_id = path.into_inner();

    let listing = sqlx::query_as::<_, crate::models::PartnerListing>(
        "SELECT * FROM partner_listings WHERE id = $1
         AND listing_status IN ('submitted','in_review')",
    )
    .bind(listing_id)
    .fetch_optional(pool.get_ref())
    .await;
    let listing = match listing {
        Ok(Some(l)) => l,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Listing not found or not awaiting review"
            }))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };

    let price = match listing.price_per_day {
        Some(p) => p,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Listing has no price set — host must complete the Pricing step before approval"
            }))
        }
    };
    let location = match listing.location.clone() {
        Some(l) if !l.is_empty() => l,
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Listing has no location set — host must complete the Pricing step before approval"
            }))
        }
    };

    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };

    let car_id = Uuid::new_v4();
    let empty_features: Vec<String> = vec![];
    let insert_car = sqlx::query(
        r#"INSERT INTO cars
            (id, host_id, make, model, year, color, plate_number, description,
             price_per_day, location, latitude, longitude, photos, features,
             status, seats, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, 'active', 5, NOW(), NOW())"#,
    )
    .bind(car_id)
    .bind(listing.user_id)
    .bind(&listing.brand)
    .bind(&listing.model)
    .bind(listing.year)
    .bind(&listing.color)
    .bind(&listing.plate_number)
    .bind(listing.description.clone().unwrap_or_default())
    .bind(price)
    .bind(&location)
    .bind(listing.latitude)
    .bind(listing.longitude)
    .bind(&listing.photos)
    .bind(&empty_features)
    .execute(&mut *tx)
    .await;
    if let Err(e) = insert_car {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": format!("Insert car failed: {e}")}));
    }

    let update_listing = sqlx::query(
        "UPDATE partner_listings
         SET listing_status = 'approved', rejection_reason = NULL,
             car_id = $2, updated_at = NOW()
         WHERE id = $1",
    )
    .bind(listing_id)
    .bind(car_id)
    .execute(&mut *tx)
    .await;
    if let Err(e) = update_listing {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()}));
    }

    let _ = sqlx::query(
        "UPDATE users SET role = 'host', updated_at = NOW()
         WHERE id = $1 AND role <> 'host' AND role <> 'admin'",
    )
    .bind(listing.user_id)
    .execute(&mut *tx)
    .await;

    if let Err(e) = tx.commit().await {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()}));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Listing approved", "car_id": car_id
    }))
}

/// POST /api/admin/partner-listings/{id}/reject
#[utoipa::path(
    post,
    path = "/api/admin/partner-listings/{id}/reject",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Partner listing ID")),
    request_body = AdminRejectBody,
    responses(
        (status = 200, description = "Listing rejected"),
        (status = 404, description = "Listing not found"),
    ),
)]
pub async fn reject_partner_listing(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<AdminRejectBody>,
) -> HttpResponse {
    if let Err(resp) = require_admin(&req) {
        return resp;
    }

    let listing_id = path.into_inner();
    let result = sqlx::query(
        "UPDATE partner_listings
         SET listing_status = 'rejected', rejection_reason = $2, updated_at = NOW()
         WHERE id = $1",
    )
    .bind(listing_id)
    .bind(body.reason.clone())
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            HttpResponse::Ok().json(serde_json::json!({"message": "Listing rejected"}))
        }
        Ok(_) => HttpResponse::NotFound()
            .json(serde_json::json!({"error": "Listing not found"})),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()})),
    }
}
