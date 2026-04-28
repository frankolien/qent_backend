use actix_web::{web, HttpResponse};
use sqlx::PgPool;

/// GET /health — Health check for monitoring/uptime services
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "{ status: healthy, database: connected, version }"),
        (status = 503, description = "Database disconnected"),
    ),
)]
pub async fn health_check(pool: web::Data<PgPool>) -> HttpResponse {
    // Check DB connection
    let db_ok = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool.get_ref())
        .await
        .is_ok();

    if db_ok {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "healthy",
            "database": "connected",
            "version": env!("CARGO_PKG_VERSION"),
        }))
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "unhealthy",
            "database": "disconnected",
        }))
    }
}
