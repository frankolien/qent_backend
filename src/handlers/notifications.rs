use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::{Claims, Notification};

#[derive(Debug, Deserialize, ToSchema)]
pub struct BulkDeleteRequest {
    pub ids: Vec<Uuid>,
}

/// GET /api/notifications — List the user's most recent 50 notifications (newest first)
#[utoipa::path(
    get,
    path = "/api/notifications",
    tag = "Notifications",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Notifications", body = Vec<Notification>),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn get_notifications(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let result = sqlx::query_as::<_, Notification>(
        "SELECT * FROM notifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT 50",
    )
    .bind(claims.sub)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(notifications) => HttpResponse::Ok().json(notifications),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// POST /api/notifications/{id}/read — Mark a single notification as read
#[utoipa::path(
    post,
    path = "/api/notifications/{id}/read",
    tag = "Notifications",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Notification ID")),
    responses(
        (status = 200, description = "Marked as read"),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn mark_read(
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

    let notification_id = path.into_inner();

    let _ = sqlx::query("UPDATE notifications SET is_read = true WHERE id = $1 AND user_id = $2")
        .bind(notification_id)
        .bind(claims.sub)
        .execute(pool.get_ref())
        .await;

    HttpResponse::Ok().json(serde_json::json!({"message": "Marked as read"}))
}

/// POST /api/notifications/read-all — Mark every notification for the user as read
#[utoipa::path(
    post,
    path = "/api/notifications/read-all",
    tag = "Notifications",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "All marked as read"),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn mark_all_read(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let _ = sqlx::query("UPDATE notifications SET is_read = true WHERE user_id = $1")
        .bind(claims.sub)
        .execute(pool.get_ref())
        .await;

    HttpResponse::Ok().json(serde_json::json!({"message": "All marked as read"}))
}

/// DELETE /api/notifications/{id} — Delete a single notification
#[utoipa::path(
    delete,
    path = "/api/notifications/{id}",
    tag = "Notifications",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Notification ID")),
    responses(
        (status = 200, description = "Deleted"),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn delete_notification(
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

    let notification_id = path.into_inner();

    let result = sqlx::query("DELETE FROM notifications WHERE id = $1 AND user_id = $2")
        .bind(notification_id)
        .bind(claims.sub)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"message": "Deleted"})),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// POST /api/notifications/delete-bulk — Delete multiple notifications by IDs in one call
#[utoipa::path(
    post,
    path = "/api/notifications/delete-bulk",
    tag = "Notifications",
    security(("bearer_auth" = [])),
    request_body = BulkDeleteRequest,
    responses(
        (status = 200, description = "{ deleted: <count> }"),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn delete_bulk(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<BulkDeleteRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    if body.ids.is_empty() {
        return HttpResponse::Ok().json(serde_json::json!({"deleted": 0}));
    }

    let result = sqlx::query("DELETE FROM notifications WHERE user_id = $1 AND id = ANY($2)")
        .bind(claims.sub)
        .bind(&body.ids)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(r) => HttpResponse::Ok()
            .json(serde_json::json!({"deleted": r.rows_affected()})),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}
