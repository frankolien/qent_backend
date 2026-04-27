use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::Claims;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct StoryResponse {
    pub id: Uuid,
    pub host_id: Uuid,
    pub car_id: Option<Uuid>,
    pub image_url: String,
    pub caption: String,
    pub created_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
    // Joined
    pub host_name: String,
    pub host_photo: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateStoryRequest {
    pub image_url: String,
    pub car_id: Option<Uuid>,
    pub caption: Option<String>,
}

/// GET /api/stories - Get all active (non-expired) stories
pub async fn get_stories(_req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let result = sqlx::query_as::<_, StoryResponse>(
        r#"SELECT
            s.id, s.host_id, s.car_id, s.image_url, s.caption,
            s.created_at, s.expires_at,
            u.full_name AS host_name,
            COALESCE(u.profile_photo_url, '') AS host_photo
        FROM stories s
        JOIN users u ON u.id = s.host_id
        WHERE s.expires_at > NOW()
        ORDER BY s.created_at DESC"#,
    )
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(stories) => HttpResponse::Ok().json(stories),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// POST /api/stories - Create a story (host only)
pub async fn create_story(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<CreateStoryRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let user_id = claims.sub;

    // Verify user is a host
    let is_host = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1 AND role = 'Host')",
    )
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(false);

    if !is_host {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Only hosts can create stories"}));
    }

    let story_id = Uuid::new_v4();

    let insert_result = sqlx::query(
        r#"INSERT INTO stories (id, host_id, car_id, image_url, caption, created_at, expires_at)
        VALUES ($1, $2, $3, $4, $5, NOW(), NOW() + INTERVAL '24 hours')"#,
    )
    .bind(story_id)
    .bind(user_id)
    .bind(body.car_id)
    .bind(&body.image_url)
    .bind(body.caption.as_deref().unwrap_or(""))
    .execute(pool.get_ref())
    .await;

    if let Err(e) = insert_result {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()}));
    }

    let result = sqlx::query_as::<_, StoryResponse>(
        r#"SELECT
            s.id, s.host_id, s.car_id, s.image_url, s.caption,
            s.created_at, s.expires_at,
            u.full_name AS host_name,
            COALESCE(u.profile_photo_url, '') AS host_photo
        FROM stories s
        JOIN users u ON u.id = s.host_id
        WHERE s.id = $1"#,
    )
    .bind(story_id)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(story) => HttpResponse::Created().json(story),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// DELETE /api/stories/{id} - Delete own story
pub async fn delete_story(
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

    let story_id = path.into_inner();
    let user_id = claims.sub;

    let result = sqlx::query("DELETE FROM stories WHERE id = $1 AND host_id = $2")
        .bind(story_id)
        .bind(user_id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            HttpResponse::Ok().json(serde_json::json!({"message": "Story deleted"}))
        }
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({"error": "Story not found"})),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}
