use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use chrono::NaiveDate;
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use validator::Validate;

use crate::models::{
    Car, CarSearchQuery, CarStatus, Claims, CreateCarRequest, HomepageQuery, UpdateCarRequest,
    UserRole,
};

pub async fn create_car(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<CreateCarRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    if claims.role != UserRole::Host && claims.role != UserRole::Admin {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Only hosts can list cars"}));
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
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
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
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

pub async fn search_cars(
    pool: web::Data<PgPool>,
    query: web::Query<CarSearchQuery>,
) -> HttpResponse {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    // Build ORDER BY based on sort_by param
    let order_clause = match query.sort_by.as_deref() {
        Some("price_asc") => "c.price_per_day ASC, c.created_at DESC",
        Some("price_desc") => "c.price_per_day DESC, c.created_at DESC",
        Some("newest") => "c.created_at DESC",
        Some("rating") => "COALESCE(rs.avg_rating, 0.0) DESC, c.created_at DESC",
        Some("distance") => {
            if query.latitude.is_some() && query.longitude.is_some() {
                // Haversine sort — approximate for ORDER BY
                "distance_km ASC NULLS LAST, c.created_at DESC"
            } else {
                "COALESCE(rs.avg_rating, 0.0) DESC, c.created_at DESC"
            }
        }
        _ => "COALESCE(rs.avg_rating, 0.0) DESC, c.created_at DESC",
    };

    // If sorting by distance, add a distance column
    let distance_select = if query.sort_by.as_deref() == Some("distance")
        && query.latitude.is_some()
        && query.longitude.is_some()
    {
        format!(
            ", (6371 * acos(cos(radians({})) * cos(radians(c.latitude)) * cos(radians(c.longitude) - radians({})) + sin(radians({})) * sin(radians(c.latitude)))) as distance_km",
            query.latitude.unwrap(), query.longitude.unwrap(), query.latitude.unwrap()
        )
    } else {
        String::new()
    };

    let sql = format!(
        r#"SELECT c.*,
            COALESCE(rs.avg_rating, 0.0) as rating,
            COALESCE(rs.trip_count, 0) as trip_count,
            u.full_name as host_name
            {}
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
        ORDER BY {}
        LIMIT $10 OFFSET $11"#,
        distance_select, order_clause
    );

    let result = sqlx::query_as::<_, Car>(&sql)
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
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

pub async fn get_host_cars(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
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
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
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
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
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
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}));
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
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

pub async fn deactivate_car(
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
        Ok(_) => HttpResponse::NotFound()
            .json(serde_json::json!({"error": "Car not found or not authorized"})),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

#[derive(Debug, Serialize, FromRow)]
struct BookedDateRange {
    start_date: NaiveDate,
    end_date: NaiveDate,
}

pub async fn get_booked_dates(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let car_id = path.into_inner();

    let result = sqlx::query_as::<_, BookedDateRange>(
        r#"SELECT start_date, end_date FROM bookings
        WHERE car_id = $1 AND status IN ('approved', 'confirmed', 'active')
        ORDER BY start_date"#,
    )
    .bind(car_id)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(ranges) => HttpResponse::Ok().json(ranges),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// GET /api/cars/homepage — Returns categorized car sections for the home feed.
/// Accepts optional ?latitude=&longitude= for distance-based "Nearby" sorting.
/// If the request is authenticated, "Recommended" is personalized based on booking history.
pub async fn get_homepage(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    query: web::Query<HomepageQuery>,
) -> HttpResponse {
    // Try to get user ID from auth token (optional — homepage is public)
    let user_id = req.extensions().get::<Claims>().map(|c| c.sub);

    // Base query fragment for active cars with rating + host name
    let all_cars = sqlx::query_as::<_, Car>(
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
        ORDER BY c.created_at DESC"#,
    )
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    // --- Best Cars: top rated (rating >= 4.0, sorted by rating desc) ---
    let mut best_cars: Vec<&Car> = all_cars
        .iter()
        .filter(|c| c.rating.unwrap_or(0.0) >= 4.0)
        .collect();
    best_cars.sort_by(|a, b| {
        b.rating
            .unwrap_or(0.0)
            .partial_cmp(&a.rating.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let best_cars: Vec<&Car> = best_cars.into_iter().take(10).collect();

    // --- Popular Cars: most booked (trip_count desc) ---
    let mut popular_cars: Vec<&Car> = all_cars
        .iter()
        .filter(|c| c.trip_count.unwrap_or(0) > 0)
        .collect();
    popular_cars.sort_by(|a, b| b.trip_count.unwrap_or(0).cmp(&a.trip_count.unwrap_or(0)));
    let popular_cars: Vec<&Car> = popular_cars.into_iter().take(10).collect();

    // --- Nearby: sorted by distance from user's coordinates ---
    let nearby_cars: Vec<&Car> = if let (Some(lat), Some(lng)) = (query.latitude, query.longitude) {
        let mut with_dist: Vec<(&Car, f64)> = all_cars
            .iter()
            .filter_map(|c| {
                if let (Some(clat), Some(clng)) = (c.latitude, c.longitude) {
                    let dist = haversine_km(lat, lng, clat, clng);
                    Some((c, dist))
                } else {
                    None
                }
            })
            .collect();
        with_dist.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        with_dist.into_iter().take(10).map(|(c, _)| c).collect()
    } else {
        // No coordinates — just return newest cars
        all_cars.iter().take(10).collect()
    };

    // --- Recommended for You: personalized or fallback to recent ---
    let recommended_cars: Vec<&Car> = if let Some(uid) = user_id {
        // Get makes and avg price from user's past bookings
        let prefs = sqlx::query_as::<_, (Option<String>, Option<f64>)>(
            r#"SELECT c.make, AVG(c.price_per_day)::double precision
               FROM bookings b
               JOIN cars c ON c.id = b.car_id
               WHERE b.renter_id = $1 AND b.status IN ('completed', 'active', 'confirmed')
               GROUP BY c.make
               ORDER BY COUNT(*) DESC
               LIMIT 3"#,
        )
        .bind(uid)
        .fetch_all(pool.get_ref())
        .await
        .unwrap_or_default();

        if prefs.is_empty() {
            // No booking history — show newest cars
            all_cars.iter().take(10).collect()
        } else {
            let fav_makes: Vec<String> = prefs.iter().filter_map(|(m, _)| m.clone()).collect();
            let avg_price: f64 = prefs.iter().filter_map(|(_, p)| *p).sum::<f64>()
                / prefs.iter().filter(|(_, p)| p.is_some()).count().max(1) as f64;
            let price_range = avg_price * 0.5..=avg_price * 1.5;

            let mut recs: Vec<&Car> = all_cars
                .iter()
                .filter(|c| {
                    fav_makes.iter().any(|m| m.eq_ignore_ascii_case(&c.make))
                        || price_range.contains(&c.price_per_day)
                })
                .collect();
            // Sort by relevance: matching make first, then by rating
            recs.sort_by(|a, b| {
                let a_make = fav_makes.iter().any(|m| m.eq_ignore_ascii_case(&a.make));
                let b_make = fav_makes.iter().any(|m| m.eq_ignore_ascii_case(&b.make));
                b_make.cmp(&a_make).then(
                    b.rating
                        .unwrap_or(0.0)
                        .partial_cmp(&a.rating.unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal),
                )
            });
            if recs.is_empty() {
                all_cars.iter().take(10).collect()
            } else {
                recs.into_iter().take(10).collect()
            }
        }
    } else {
        // Not logged in — show newest
        all_cars.iter().take(10).collect()
    };

    HttpResponse::Ok().json(serde_json::json!({
        "recommended": recommended_cars,
        "best_cars": best_cars,
        "nearby": nearby_cars,
        "popular": popular_cars,
    }))
}

/// Haversine distance in kilometers between two lat/lng points
fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0; // Earth radius in km
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    r * c
}
