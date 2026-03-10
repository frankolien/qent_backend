use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::models::{
    AuthResponse, Claims, SignInRequest, SignUpRequest, UpdateProfileRequest, UserPublic,
    VerificationStatus, VerifyIdentityRequest,
};
use crate::services::AppConfig;

pub async fn sign_up(
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    body: web::Json<SignUpRequest>,
) -> HttpResponse {
    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({"errors": e.to_string()}));
    }

    let existing = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"
    )
    .bind(&body.email)
    .fetch_one(pool.get_ref())
    .await;

    if let Ok(true) = existing {
        return HttpResponse::Conflict().json(serde_json::json!({"error": "Email already registered"}));
    }

    let password_hash = match hash(&body.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to hash password"})),
    };

    let id = Uuid::new_v4();
    let now = Utc::now().naive_utc();

    let country = body.country.clone().unwrap_or_else(|| "Nigeria".to_string());

    let result = sqlx::query(
        r#"INSERT INTO users (id, email, phone, password_hash, full_name, role, verification_status, wallet_balance, is_active, country, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 0.0, true, $8, $9, $9)"#,
    )
    .bind(id)
    .bind(&body.email)
    .bind(&body.phone)
    .bind(&password_hash)
    .bind(&body.full_name)
    .bind(crate::models::UserRole::Renter)
    .bind(VerificationStatus::Pending)
    .bind(&country)
    .bind(now)
    .execute(pool.get_ref())
    .await;

    if let Err(_e) = result {
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Registration failed"}));
    }

    let claims = Claims {
        sub: id,
        role: crate::models::UserRole::Renter,
        exp: (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .unwrap();

    HttpResponse::Created().json(AuthResponse {
        token,
        user: UserPublic {
            id,
            email: body.email.clone(),
            phone: body.phone.clone(),
            full_name: body.full_name.clone(),
            role: body.role.clone(),
            profile_photo_url: None,
            verification_status: VerificationStatus::Pending,
            wallet_balance: 0.0,
            is_active: true,
            country: body.country.clone().unwrap_or_else(|| "Nigeria".to_string()),
            created_at: now,
        },
    })
}

pub async fn sign_in(
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    body: web::Json<SignInRequest>,
) -> HttpResponse {
    let user = sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM users WHERE email = $1 AND is_active = true",
    )
    .bind(&body.email)
    .fetch_optional(pool.get_ref())
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Invalid credentials"}))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Internal server error"}))
        }
    };

    if !verify(&body.password, &user.password_hash).unwrap_or(false) {
        return HttpResponse::Unauthorized()
            .json(serde_json::json!({"error": "Invalid credentials"}));
    }

    let claims = Claims {
        sub: user.id,
        role: user.role.clone(),
        exp: (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .unwrap();

    HttpResponse::Ok().json(AuthResponse {
        token,
        user: user.into(),
    })
}

pub async fn get_profile(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = req.extensions().get::<Claims>().cloned();
    let claims = match claims {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    let user = sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM users WHERE id = $1",
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await;

    match user {
        Ok(Some(u)) => HttpResponse::Ok().json(UserPublic::from(u)),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({"error": "User not found"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn update_profile(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<UpdateProfileRequest>,
) -> HttpResponse {
    let claims = req.extensions().get::<Claims>().cloned();
    let claims = match claims {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({"errors": e.to_string()}));
    }

    let result = sqlx::query(
        r#"UPDATE users SET
            full_name = COALESCE($1, full_name),
            phone = COALESCE($2, phone),
            profile_photo_url = COALESCE($3, profile_photo_url),
            updated_at = NOW()
        WHERE id = $4"#,
    )
    .bind(&body.full_name)
    .bind(&body.phone)
    .bind(&body.profile_photo_url)
    .bind(claims.sub)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"message": "Profile updated"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn verify_identity(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<VerifyIdentityRequest>,
) -> HttpResponse {
    let claims = req.extensions().get::<Claims>().cloned();
    let claims = match claims {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    let result = sqlx::query(
        r#"UPDATE users SET
            drivers_license_url = $1,
            id_card_url = $2,
            verification_status = $3,
            updated_at = NOW()
        WHERE id = $4"#,
    )
    .bind(&body.drivers_license_url)
    .bind(&body.id_card_url)
    .bind(VerificationStatus::Pending)
    .bind(claims.sub)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"message": "Identity documents submitted for verification"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}
