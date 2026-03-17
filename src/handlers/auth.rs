use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::models::{
    AuthResponse, AuthResponseWithRefresh, Claims, ForgotPasswordRequest, RefreshTokenRequest,
    ResetPasswordRequest, SignInRequest, SignUpRequest, UpdateProfileRequest, User, UserPublic,
    VerificationStatus, VerifyIdentityRequest,
};
use crate::services::email::EmailService;
use crate::services::AppConfig;

fn generate_refresh_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..64).map(|_| {
        let idx = rng.gen_range(0..36);
        if idx < 10 { (b'0' + idx) as char } else { (b'a' + idx - 10) as char }
    }).collect()
}

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

    // Generate refresh token
    let refresh = generate_refresh_token();
    let _ = sqlx::query("UPDATE users SET refresh_token = $1 WHERE id = $2")
        .bind(&refresh)
        .bind(id)
        .execute(pool.get_ref())
        .await;

    HttpResponse::Created().json(AuthResponseWithRefresh {
        token,
        refresh_token: refresh,
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
        "SELECT * FROM users WHERE LOWER(email) = LOWER($1) AND is_active = true",
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

    // Generate refresh token
    let refresh = generate_refresh_token();
    let _ = sqlx::query("UPDATE users SET refresh_token = $1 WHERE id = $2")
        .bind(&refresh)
        .bind(user.id)
        .execute(pool.get_ref())
        .await;

    HttpResponse::Ok().json(AuthResponseWithRefresh {
        token,
        refresh_token: refresh,
        user: user.into(),
    })
}

/// POST /api/auth/refresh — Get new JWT using refresh token
pub async fn refresh_token(
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    body: web::Json<RefreshTokenRequest>,
) -> HttpResponse {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE refresh_token = $1 AND is_active = true",
    )
    .bind(&body.refresh_token)
    .fetch_optional(pool.get_ref())
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid refresh token"})),
    };

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

    // Rotate refresh token
    let new_refresh = generate_refresh_token();
    let _ = sqlx::query("UPDATE users SET refresh_token = $1 WHERE id = $2")
        .bind(&new_refresh)
        .bind(user.id)
        .execute(pool.get_ref())
        .await;

    HttpResponse::Ok().json(AuthResponseWithRefresh {
        token,
        refresh_token: new_refresh,
        user: user.into(),
    })
}

/// POST /api/auth/forgot-password — Send password reset email
pub async fn forgot_password(
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    body: web::Json<ForgotPasswordRequest>,
) -> HttpResponse {
    // Always return success to prevent email enumeration
    let success_msg = serde_json::json!({"message": "If that email is registered, a reset link has been sent."});

    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE LOWER(email) = LOWER($1) AND is_active = true",
    )
    .bind(&body.email)
    .fetch_optional(pool.get_ref())
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Ok().json(&success_msg),
    };

    // Generate reset token (64 hex chars)
    let token = generate_refresh_token();
    let expires = Utc::now() + chrono::Duration::minutes(30);

    // Invalidate old tokens
    let _ = sqlx::query("UPDATE password_reset_tokens SET used = true WHERE user_id = $1 AND used = false")
        .bind(user.id)
        .execute(pool.get_ref())
        .await;

    // Store token
    let _ = sqlx::query(
        "INSERT INTO password_reset_tokens (user_id, token, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(user.id)
    .bind(&token)
    .bind(expires.naive_utc())
    .execute(pool.get_ref())
    .await;

    // Send email
    let email_service = EmailService::new(config.resend_api_key.clone());
    let reset_message = format!(
        "You requested a password reset. Use this code in the app: {}\n\nThis code expires in 30 minutes.",
        &token[..8] // Use first 8 chars as user-facing code
    );
    email_service
        .send_status_email(&user.email, &user.full_name, "Qent", "Password Reset", &reset_message)
        .await;

    HttpResponse::Ok().json(&success_msg)
}

/// POST /api/auth/reset-password — Reset password with token
pub async fn reset_password(
    pool: web::Data<PgPool>,
    body: web::Json<ResetPasswordRequest>,
) -> HttpResponse {
    if body.new_password.len() < 6 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Password must be at least 6 characters"}));
    }

    // Find valid token (match by prefix for short codes, or full token)
    let token_record = sqlx::query_as::<_, (Uuid, Uuid)>(
        r#"SELECT id, user_id FROM password_reset_tokens
           WHERE (token = $1 OR LEFT(token, 8) = $1)
           AND used = false AND expires_at > NOW()
           ORDER BY created_at DESC LIMIT 1"#,
    )
    .bind(&body.token)
    .fetch_optional(pool.get_ref())
    .await;

    let (token_id, user_id) = match token_record {
        Ok(Some(r)) => r,
        _ => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid or expired reset code"})),
    };

    let password_hash = match hash(&body.new_password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to hash password"})),
    };

    // Update password
    let _ = sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(&password_hash)
        .bind(user_id)
        .execute(pool.get_ref())
        .await;

    // Mark token as used
    let _ = sqlx::query("UPDATE password_reset_tokens SET used = true WHERE id = $1")
        .bind(token_id)
        .execute(pool.get_ref())
        .await;

    HttpResponse::Ok().json(serde_json::json!({"message": "Password reset successfully"}))
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

/// Get any user's public profile by ID (for displaying names/photos in chat, car listings, etc.)
pub async fn get_user_public(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let user_id = path.into_inner();

    let result = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool.get_ref())
        .await;

    match result {
        Ok(Some(u)) => {
            let public = serde_json::json!({
                "id": u.id,
                "full_name": u.full_name,
                "profile_photo_url": u.profile_photo_url,
                "role": u.role,
            });
            HttpResponse::Ok().json(public)
        }
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
