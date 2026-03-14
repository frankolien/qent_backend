use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::models::{Car, CarSearchQuery, CarStatus, Claims, CreateCarRequest, UpdateCarRequest, UserRole};

pub async fn create_car(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<CreateCarRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    if claims.role != UserRole::Host && claims.role != UserRole::Admin {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": "Only hosts can list cars"}));
    }

    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({"errors": e.to_string()}));
    }

    let id = Uuid::new_v4();
    let features = body.features.clone().unwrap_or_default();

    let seats = body.seats.unwrap_or(5);

    let result = sqlx::query_as::<_, Car>(
        r#"WITH inserted AS (
            INSERT INTO cars (id, host_id, make, model, year, color, plate_number, description,
                price_per_day, location, latitude, longitude, photos, features, status, seats,
                available_from, available_to, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, NOW(), NOW())
            RETURNING *
        )
        SELECT inserted.*, 0.0::double precision as rating, 0::bigint as trip_count,
            u.full_name as host_name
        FROM inserted
        LEFT JOIN users u ON u.id = inserted.host_id"#,
    )
    .bind(id)
    .bind(claims.sub)
    .bind(&body.make)
    .bind(&body.model)
    .bind(body.year)
    .bind(&body.color)
    .bind(&body.plate_number)
    .bind(&body.description)
    .bind(body.price_per_day)
    .bind(&body.location)
    .bind(body.latitude)
    .bind(body.longitude)
    .bind(&body.photos)
    .bind(&features)
    .bind(CarStatus::PendingApproval)
    .bind(seats)
    .bind(body.available_from)
    .bind(body.available_to)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(car) => HttpResponse::Created().json(car),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn get_car(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let car_id = path.into_inner();

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
        WHERE c.id = $1"#,
    )
        .bind(car_id)
        .fetch_optional(pool.get_ref())
        .await;

    match result {
        Ok(Some(car)) => HttpResponse::Ok().json(car),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({"error": "Car not found"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn search_cars(
    pool: web::Data<PgPool>,
    query: web::Query<CarSearchQuery>,
) -> HttpResponse {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

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
        WHERE c.status = 'active'
        AND ($1::text IS NULL OR LOWER(c.location) LIKE LOWER('%' || $1 || '%'))
        AND ($2::double precision IS NULL OR c.price_per_day >= $2)
        AND ($3::double precision IS NULL OR c.price_per_day <= $3)
        AND ($4::text IS NULL OR LOWER(c.make) = LOWER($4))
        AND ($5::text IS NULL OR LOWER(c.model) = LOWER($5))
        AND ($6::date IS NULL OR c.available_from IS NULL OR c.available_from <= $6)
        AND ($7::date IS NULL OR c.available_to IS NULL OR c.available_to >= $7)
        AND ($8::text IS NULL OR LOWER(c.color) = LOWER($8))
        AND ($9::integer IS NULL OR c.seats = $9)
        ORDER BY COALESCE(rs.avg_rating, 0.0) DESC, c.created_at DESC
        LIMIT $10 OFFSET $11"#,
    )
    .bind(&query.location)
    .bind(query.min_price)
    .bind(query.max_price)
    .bind(&query.make)
    .bind(&query.model)
    .bind(query.start_date)
    .bind(query.end_date)
    .bind(&query.color)
    .bind(query.seats)
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(cars) => HttpResponse::Ok().json(cars),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn get_host_cars(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

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
        WHERE c.host_id = $1
        ORDER BY c.created_at DESC"#,
    )
    .bind(claims.sub)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(cars) => HttpResponse::Ok().json(cars),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn update_car(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateCarRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    let car_id = path.into_inner();

    // Verify ownership
    let car = sqlx::query_scalar::<_, Uuid>("SELECT host_id FROM cars WHERE id = $1")
        .bind(car_id)
        .fetch_optional(pool.get_ref())
        .await;

    match &car {
        Ok(Some(host_id)) if *host_id != claims.sub && claims.role != UserRole::Admin => {
            return HttpResponse::Forbidden().json(serde_json::json!({"error": "Not your car"}));
        }
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({"error": "Car not found"}));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}));
        }
        _ => {}
    }

    let result = sqlx::query_as::<_, Car>(
        r#"WITH updated AS (
            UPDATE cars SET
                description = COALESCE($1, description),
                price_per_day = COALESCE($2, price_per_day),
                location = COALESCE($3, location),
                latitude = COALESCE($4, latitude),
                longitude = COALESCE($5, longitude),
                photos = COALESCE($6, photos),
                features = COALESCE($7, features),
                available_from = COALESCE($8, available_from),
                available_to = COALESCE($9, available_to),
                updated_at = NOW()
            WHERE id = $10
            RETURNING *
        )
        SELECT updated.*,
            COALESCE(rs.avg_rating, 0.0) as rating,
            COALESCE(rs.trip_count, 0) as trip_count,
            usr.full_name as host_name
        FROM updated
        LEFT JOIN (
            SELECT b.car_id,
                   AVG(r.rating)::double precision as avg_rating,
                   COUNT(DISTINCT b.id) as trip_count
            FROM reviews r
            JOIN bookings b ON r.booking_id = b.id
            GROUP BY b.car_id
        ) rs ON rs.car_id = updated.id
        LEFT JOIN users usr ON usr.id = updated.host_id"#,
    )
    .bind(&body.description)
    .bind(body.price_per_day)
    .bind(&body.location)
    .bind(body.latitude)
    .bind(body.longitude)
    .bind(&body.photos)
    .bind(&body.features)
    .bind(body.available_from)
    .bind(body.available_to)
    .bind(car_id)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(car) => HttpResponse::Ok().json(car),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn deactivate_car(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    let car_id = path.into_inner();

    let result = sqlx::query(
        "UPDATE cars SET status = 'inactive', updated_at = NOW() WHERE id = $1 AND (host_id = $2 OR $3)",
    )
    .bind(car_id)
    .bind(claims.sub)
    .bind(claims.role == UserRole::Admin)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            HttpResponse::Ok().json(serde_json::json!({"message": "Car deactivated"}))
        }
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({"error": "Car not found or not authorized"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}
