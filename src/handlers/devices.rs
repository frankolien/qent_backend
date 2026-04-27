use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;

use crate::models::{Claims, RegisterDeviceTokenRequest};

pub async fn register_device_token(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<RegisterDeviceTokenRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    if !["ios", "android", "web"].contains(&body.platform.as_str()) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "platform must be one of: ios, android, web"
        }));
    }

    if body.token.trim().is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "token is required"}));
    }

    let result = sqlx::query(
        "INSERT INTO device_tokens (user_id, token, platform)
         VALUES ($1, $2, $3)
         ON CONFLICT (token) DO UPDATE
            SET user_id = EXCLUDED.user_id,
                platform = EXCLUDED.platform,
                last_seen_at = CURRENT_TIMESTAMP",
    )
    .bind(claims.sub)
    .bind(&body.token)
    .bind(&body.platform)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"message": "Device registered"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn unregister_device_token(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    let token = path.into_inner();

    let _ = sqlx::query("DELETE FROM device_tokens WHERE token = $1 AND user_id = $2")
        .bind(&token)
        .bind(claims.sub)
        .execute(pool.get_ref())
        .await;

    HttpResponse::Ok().json(serde_json::json!({"message": "Device unregistered"}))
}
