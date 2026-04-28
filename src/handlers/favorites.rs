use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Car, Claims};

#[utoipa::path(
    post,
    path = "/api/favorites/{id}",
    tag = "Favorites",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Car ID to favorite/unfavorite")),
    responses(
        (status = 200, description = "Toggled. Response: {\"favorited\": bool}"),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn toggle_favorite(
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

    let car_id = path.into_inner();

    // Check if already favorited
    let existing = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM favorites WHERE user_id = $1 AND car_id = $2)",
    )
    .bind(claims.sub)
    .bind(car_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(false);

    if existing {
        let _ = sqlx::query("DELETE FROM favorites WHERE user_id = $1 AND car_id = $2")
            .bind(claims.sub)
            .bind(car_id)
            .execute(pool.get_ref())
            .await;
        HttpResponse::Ok().json(serde_json::json!({"favorited": false}))
    } else {
        let _ = sqlx::query(
            "INSERT INTO favorites (id, user_id, car_id, created_at) VALUES ($1, $2, $3, NOW())",
        )
        .bind(Uuid::new_v4())
        .bind(claims.sub)
        .bind(car_id)
        .execute(pool.get_ref())
        .await;
        HttpResponse::Ok().json(serde_json::json!({"favorited": true}))
    }
}

#[utoipa::path(
    get,
    path = "/api/favorites",
    tag = "Favorites",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Cars favorited by the current user", body = Vec<Car>),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn get_favorites(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let result = sqlx::query_as::<_, Car>(
        r#"SELECT c.*,
            COALESCE(rs.avg_rating, 0.0) as rating,
            COALESCE(rs.trip_count, 0) as trip_count,
            u.full_name as host_name
        FROM cars c
        INNER JOIN favorites f ON f.car_id = c.id
        LEFT JOIN (
            SELECT b.car_id,
                   AVG(r.rating)::double precision as avg_rating,
                   COUNT(DISTINCT b.id) as trip_count
            FROM reviews r
            JOIN bookings b ON r.booking_id = b.id
            GROUP BY b.car_id
        ) rs ON rs.car_id = c.id
        LEFT JOIN users u ON u.id = c.host_id
        WHERE f.user_id = $1
        ORDER BY f.created_at DESC"#,
    )
    .bind(claims.sub)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(cars) => HttpResponse::Ok().json(cars),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/favorites/{id}/check",
    tag = "Favorites",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Car ID to check")),
    responses(
        (status = 200, description = "Whether the user has favorited this car. Response: {\"favorited\": bool}"),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn check_favorite(
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

    let car_id = path.into_inner();

    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM favorites WHERE user_id = $1 AND car_id = $2)",
    )
    .bind(claims.sub)
    .bind(car_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(false);

    HttpResponse::Ok().json(serde_json::json!({"favorited": exists}))
}
