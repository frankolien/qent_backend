use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Claims, UserRole};

const CURRENT_TOS_VERSION: &str = "1.0";

/// POST /api/auth/accept-terms — Record user's consent to ToS + Privacy Policy
pub async fn accept_terms(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    let _ = sqlx::query(
        r#"UPDATE users SET
            tos_accepted_at = NOW(),
            privacy_accepted_at = NOW(),
            tos_version = $1,
            updated_at = NOW()
        WHERE id = $2"#,
    )
    .bind(CURRENT_TOS_VERSION)
    .bind(claims.sub)
    .execute(pool.get_ref())
    .await;

    // Audit log
    log_action(pool.get_ref(), claims.sub, "accept_terms", &req, None).await;

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Terms accepted",
        "tos_version": CURRENT_TOS_VERSION
    }))
}

/// GET /api/auth/terms-status — Check if user has accepted current ToS
pub async fn terms_status(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    let result = sqlx::query_as::<_, (Option<String>, Option<chrono::NaiveDateTime>)>(
        "SELECT tos_version, tos_accepted_at FROM users WHERE id = $1",
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some((version, accepted_at))) => {
            let needs_acceptance = version.as_deref() != Some(CURRENT_TOS_VERSION);
            HttpResponse::Ok().json(serde_json::json!({
                "current_version": CURRENT_TOS_VERSION,
                "accepted_version": version,
                "accepted_at": accepted_at,
                "needs_acceptance": needs_acceptance
            }))
        }
        _ => HttpResponse::Ok().json(serde_json::json!({
            "current_version": CURRENT_TOS_VERSION,
            "needs_acceptance": true
        })),
    }
}

/// POST /api/account/request-deletion — Request account deletion (30-day grace period)
pub async fn request_deletion(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    // Check for active bookings
    let has_active = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM bookings WHERE (renter_id = $1 OR host_id = $1) AND status IN ('confirmed', 'active'))",
    )
    .bind(claims.sub)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(false);

    if has_active {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Cannot delete account with active bookings. Please complete or cancel them first."
        }));
    }

    // Check for pending wallet balance
    let balance = sqlx::query_scalar::<_, f64>("SELECT wallet_balance FROM users WHERE id = $1")
        .bind(claims.sub)
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0.0);

    if balance > 0.0 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Please withdraw your wallet balance before deleting your account.",
            "balance": balance
        }));
    }

    let reason = body.get("reason").and_then(|v| v.as_str()).unwrap_or("");
    let scheduled = chrono::Utc::now() + chrono::Duration::days(30);

    let _ = sqlx::query(
        r#"INSERT INTO deletion_requests (user_id, reason, scheduled_deletion_at)
        VALUES ($1, $2, $3)
        ON CONFLICT DO NOTHING"#,
    )
    .bind(claims.sub)
    .bind(reason)
    .bind(scheduled.naive_utc())
    .execute(pool.get_ref())
    .await;

    log_action(pool.get_ref(), claims.sub, "request_deletion", &req, None).await;

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Account deletion scheduled",
        "scheduled_deletion_at": scheduled.naive_utc(),
        "note": "Your account will be permanently deleted in 30 days. Log in again to cancel."
    }))
}

/// POST /api/account/cancel-deletion — Cancel a pending deletion request
pub async fn cancel_deletion(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    let updated = sqlx::query(
        "UPDATE deletion_requests SET status = 'cancelled' WHERE user_id = $1 AND status = 'pending'",
    )
    .bind(claims.sub)
    .execute(pool.get_ref())
    .await;

    match updated {
        Ok(r) if r.rows_affected() > 0 => {
            log_action(pool.get_ref(), claims.sub, "cancel_deletion", &req, None).await;
            HttpResponse::Ok().json(serde_json::json!({"message": "Account deletion cancelled"}))
        }
        _ => HttpResponse::NotFound().json(serde_json::json!({"error": "No pending deletion request found"})),
    }
}

/// GET /api/account/export — Export all user data (NDPA right of access)
pub async fn export_data(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    // Gather all user data
    let user = sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE id = $1")
        .bind(claims.sub)
        .fetch_optional(pool.get_ref())
        .await;

    let bookings = sqlx::query_as::<_, crate::models::Booking>(
        "SELECT * FROM bookings WHERE renter_id = $1 OR host_id = $1 ORDER BY created_at DESC",
    )
    .bind(claims.sub)
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    let payments = sqlx::query_as::<_, crate::models::Payment>(
        "SELECT * FROM payments WHERE payer_id = $1 ORDER BY created_at DESC",
    )
    .bind(claims.sub)
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    let reviews = sqlx::query_as::<_, crate::models::Review>(
        "SELECT * FROM reviews WHERE reviewer_id = $1 OR reviewee_id = $1 ORDER BY created_at DESC",
    )
    .bind(claims.sub)
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    log_action(pool.get_ref(), claims.sub, "data_export", &req, None).await;

    match user {
        Ok(Some(u)) => {
            let public = crate::models::UserPublic::from(u);
            HttpResponse::Ok().json(serde_json::json!({
                "user": public,
                "bookings": bookings,
                "payments": payments,
                "reviews": reviews,
                "exported_at": chrono::Utc::now().naive_utc()
            }))
        }
        _ => HttpResponse::NotFound().json(serde_json::json!({"error": "User not found"})),
    }
}

/// Helper: log an action to the audit trail
async fn log_action(
    pool: &PgPool,
    user_id: Uuid,
    action: &str,
    req: &HttpRequest,
    details: Option<serde_json::Value>,
) {
    let ip = req
        .connection_info()
        .peer_addr()
        .map(|s| s.to_string())
        .unwrap_or_default();

    let _ = sqlx::query(
        "INSERT INTO audit_log (user_id, action, ip_address, details) VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id)
    .bind(action)
    .bind(&ip)
    .bind(details)
    .execute(pool)
    .await;
}

/// Admin: GET /api/admin/audit-log — View audit trail
pub async fn admin_audit_log(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };
    if claims.role != UserRole::Admin {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": "Admin access required"}));
    }

    #[derive(sqlx::FromRow, serde::Serialize)]
    struct AuditEntry {
        id: Uuid,
        user_id: Option<Uuid>,
        action: String,
        ip_address: Option<String>,
        details: Option<serde_json::Value>,
        created_at: chrono::NaiveDateTime,
    }

    let logs = sqlx::query_as::<_, AuditEntry>(
        r#"SELECT id, user_id, action, ip_address, details, created_at
        FROM audit_log
        ORDER BY created_at DESC
        LIMIT 200"#,
    )
    .fetch_all(pool.get_ref())
    .await;

    match logs {
        Ok(entries) => HttpResponse::Ok().json(entries),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()})),
    }
}
