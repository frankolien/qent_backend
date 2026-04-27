use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::models::{Claims, CreateReviewRequest, Review, UserRatingSummary};

pub async fn create_review(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<CreateReviewRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({"errors": e.to_string()}));
    }

    // Verify completed booking
    let booking_exists = sqlx::query_scalar::<_, bool>(
        r#"SELECT EXISTS(
            SELECT 1 FROM bookings
            WHERE id = $1 AND status = 'completed'
            AND (renter_id = $2 OR host_id = $2)
        )"#,
    )
    .bind(body.booking_id)
    .bind(claims.sub)
    .fetch_one(pool.get_ref())
    .await;

    if !booking_exists.unwrap_or(false) {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Can only review after a completed booking"}));
    }

    // Check duplicate review
    let already_reviewed = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM reviews WHERE booking_id = $1 AND reviewer_id = $2)",
    )
    .bind(body.booking_id)
    .bind(claims.sub)
    .fetch_one(pool.get_ref())
    .await;

    if let Ok(true) = already_reviewed {
        return HttpResponse::Conflict()
            .json(serde_json::json!({"error": "Already reviewed this booking"}));
    }

    let id = Uuid::new_v4();
    let result = sqlx::query_as::<_, Review>(
        r#"INSERT INTO reviews (id, booking_id, reviewer_id, reviewee_id, rating, comment, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, NOW())
        RETURNING *"#,
    )
    .bind(id)
    .bind(body.booking_id)
    .bind(claims.sub)
    .bind(body.reviewee_id)
    .bind(body.rating)
    .bind(&body.comment)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(review) => HttpResponse::Created().json(review),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

pub async fn get_user_reviews(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let user_id = path.into_inner();

    let reviews = sqlx::query_as::<_, Review>(
        "SELECT * FROM reviews WHERE reviewee_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool.get_ref())
    .await;

    match reviews {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

pub async fn get_user_rating(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let user_id = path.into_inner();

    let avg = sqlx::query_scalar::<_, Option<f64>>(
        "SELECT AVG(rating::double precision) FROM reviews WHERE reviewee_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await;

    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reviews WHERE reviewee_id = $1")
        .bind(user_id)
        .fetch_one(pool.get_ref())
        .await;

    HttpResponse::Ok().json(UserRatingSummary {
        user_id,
        average_rating: avg.unwrap_or(Some(0.0)).unwrap_or(0.0),
        total_reviews: count.unwrap_or(0),
    })
}
