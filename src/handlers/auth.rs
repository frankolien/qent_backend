use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::models::{
    AppleSignInRequest, AuthResponseWithRefresh, Claims, ForgotPasswordRequest,
    GoogleSignInRequest, RefreshTokenRequest, ResetPasswordRequest, SignInRequest, SignUpRequest,
    UpdateProfileRequest, User, UserPublic, VerificationStatus, VerifyIdentityRequest,
};
use crate::services::apple_auth::verify_identity_token;
use crate::services::email::EmailService;
use crate::services::google_auth::verify_identity_token as verify_google_identity_token;
use crate::services::AppConfig;

fn generate_refresh_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..64)
        .map(|_| {
            let idx = rng.gen_range(0..36);
            if idx < 10 {
                (b'0' + idx) as char
            } else {
                (b'a' + idx - 10) as char
            }
        })
        .collect()
}

/// Sign a JWT for the given claims. Returns a 500 response on failure instead
/// of panicking — encoding can fail if the secret bytes are malformed and we
/// don't want a single bad config to kill the worker.
fn sign_token(claims: &Claims, secret: &str) -> Result<String, HttpResponse> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| {
        log::error!("JWT encoding failed: {}", e);
        HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Failed to issue session token"}))
    })
}

#[utoipa::path(
    post,
    path = "/api/auth/signup",
    tag = "Auth",
    request_body = SignUpRequest,
    responses(
        (status = 201, description = "Account created", body = AuthResponseWithRefresh),
        (status = 400, description = "Validation error or email already in use"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn sign_up(
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    body: web::Json<SignUpRequest>,
) -> HttpResponse {
    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({"errors": e.to_string()}));
    }

    let existing =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
            .bind(&body.email)
            .fetch_one(pool.get_ref())
            .await;

    if let Ok(true) = existing {
        return HttpResponse::Conflict()
            .json(serde_json::json!({"error": "Email already registered"}));
    }

    let password_hash = match hash(&body.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Failed to hash password"}))
        }
    };

    let id = Uuid::new_v4();
    let now = Utc::now().naive_utc();

    let country = body
        .country
        .clone()
        .unwrap_or_else(|| "Nigeria".to_string());

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
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Registration failed"}));
    }

    let claims = Claims {
        sub: id,
        role: crate::models::UserRole::Renter,
        exp: (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = match sign_token(&claims, &config.jwt_secret) {
        Ok(t) => t,
        Err(resp) => return resp,
    };

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
            country: body
                .country
                .clone()
                .unwrap_or_else(|| "Nigeria".to_string()),
            created_at: now,
        },
    })
}

#[utoipa::path(
    post,
    path = "/api/auth/signin",
    tag = "Auth",
    request_body = SignInRequest,
    responses(
        (status = 200, description = "Authenticated", body = AuthResponseWithRefresh),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error"),
    ),
)]
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

    let stored_hash = match &user.password_hash {
        Some(h) => h,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Invalid credentials"}));
        }
    };

    if !verify(&body.password, stored_hash).unwrap_or(false) {
        return HttpResponse::Unauthorized()
            .json(serde_json::json!({"error": "Invalid credentials"}));
    }

    let claims = Claims {
        sub: user.id,
        role: user.role.clone(),
        exp: (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = match sign_token(&claims, &config.jwt_secret) {
        Ok(t) => t,
        Err(resp) => return resp,
    };

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
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    tag = "Auth",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "New access + refresh token issued", body = AuthResponseWithRefresh),
        (status = 401, description = "Refresh token invalid or revoked"),
    ),
)]
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
        _ => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Invalid refresh token"}))
        }
    };

    let claims = Claims {
        sub: user.id,
        role: user.role.clone(),
        exp: (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = match sign_token(&claims, &config.jwt_secret) {
        Ok(t) => t,
        Err(resp) => return resp,
    };

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
#[utoipa::path(
    post,
    path = "/api/auth/forgot-password",
    tag = "Auth",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Reset email sent if account exists"),
    ),
)]
pub async fn forgot_password(
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    body: web::Json<ForgotPasswordRequest>,
) -> HttpResponse {
    // Always return success to prevent email enumeration
    let success_msg =
        serde_json::json!({"message": "If that email is registered, a reset link has been sent."});

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
    let _ = sqlx::query(
        "UPDATE password_reset_tokens SET used = true WHERE user_id = $1 AND used = false",
    )
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
        .send_status_email(
            &user.email,
            &user.full_name,
            "Qent",
            "Password Reset",
            &reset_message,
        )
        .await;

    HttpResponse::Ok().json(&success_msg)
}

/// POST /api/auth/reset-password — Reset password with token
#[utoipa::path(
    post,
    path = "/api/auth/reset-password",
    tag = "Auth",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password updated"),
        (status = 400, description = "Invalid or expired token"),
    ),
)]
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
        _ => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "Invalid or expired reset code"}))
        }
    };

    let password_hash = match hash(&body.new_password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Failed to hash password"}))
        }
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

#[utoipa::path(
    get,
    path = "/api/profile",
    tag = "Auth",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current authenticated user", body = UserPublic),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn get_profile(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = req.extensions().get::<Claims>().cloned();
    let claims = match claims {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let user = sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE id = $1")
        .bind(claims.sub)
        .fetch_optional(pool.get_ref())
        .await;

    match user {
        Ok(Some(u)) => HttpResponse::Ok().json(UserPublic::from(u)),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({"error": "User not found"})),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// Get any user's public profile by ID (for displaying names/photos in chat, car listings, etc.)
#[utoipa::path(
    get,
    path = "/api/users/{id}",
    tag = "Users",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "Public profile", body = UserPublic),
        (status = 404, description = "User not found"),
    ),
)]
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
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/profile",
    tag = "Auth",
    security(("bearer_auth" = [])),
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Updated profile", body = UserPublic),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn update_profile(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<UpdateProfileRequest>,
) -> HttpResponse {
    let claims = req.extensions().get::<Claims>().cloned();
    let claims = match claims {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
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
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/profile/verify-identity",
    tag = "Auth",
    security(("bearer_auth" = [])),
    request_body = VerifyIdentityRequest,
    responses(
        (status = 200, description = "Verification submitted, status now pending review"),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn verify_identity(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<VerifyIdentityRequest>,
) -> HttpResponse {
    let claims = req.extensions().get::<Claims>().cloned();
    let claims = match claims {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
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
        Ok(_) => HttpResponse::Ok()
            .json(serde_json::json!({"message": "Identity documents submitted for verification"})),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// POST /api/auth/signin/apple
///
/// Accepts an Apple `identityToken` (JWT signed by Apple), verifies it against
/// Apple's JWKS, and either creates a new user (first sign-in) or signs in an
/// existing one. Returns our standard JWT + refresh token.
///
/// Apple only sends `fullName` on the first sign-in ever for a given user/team,
/// so the mobile app must pass it on the first call. Subsequent calls can omit it.
#[utoipa::path(
    post,
    path = "/api/auth/signin/apple",
    tag = "Auth",
    request_body = AppleSignInRequest,
    responses(
        (status = 200, description = "Authenticated via Apple", body = AuthResponseWithRefresh),
        (status = 401, description = "Invalid Apple identity token"),
    ),
)]
pub async fn sign_in_with_apple(
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    body: web::Json<AppleSignInRequest>,
) -> HttpResponse {
    let claims = match verify_identity_token(&body.identity_token, &config.apple_bundle_id).await {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Apple identityToken verification failed: {e}");
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Invalid Apple identity token"}));
        }
    };

    let apple_sub = claims.sub;
    let apple_email = claims.email.or_else(|| body.email.clone());

    // 1) Match by apple_id first (the stable identifier)
    let existing =
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE apple_id = $1 AND is_active = true")
            .bind(&apple_sub)
            .fetch_optional(pool.get_ref())
            .await;

    let user = match existing {
        Ok(Some(u)) => u,
        Ok(None) => {
            // 2) Not linked yet — try to link to an existing email account,
            //    otherwise create a brand new user.
            let by_email = if let Some(ref email) = apple_email {
                sqlx::query_as::<_, User>(
                    "SELECT * FROM users WHERE LOWER(email) = LOWER($1) AND is_active = true",
                )
                .bind(email)
                .fetch_optional(pool.get_ref())
                .await
                .ok()
                .flatten()
            } else {
                None
            };

            if let Some(u) = by_email {
                let _ =
                    sqlx::query("UPDATE users SET apple_id = $1, updated_at = NOW() WHERE id = $2")
                        .bind(&apple_sub)
                        .bind(u.id)
                        .execute(pool.get_ref())
                        .await;
                User {
                    apple_id: Some(apple_sub.clone()),
                    ..u
                }
            } else {
                // Create a new user. Email may be absent on relay-only first sign-ins;
                // fall back to a synthetic address so our UNIQUE email constraint holds.
                let email = apple_email
                    .clone()
                    .unwrap_or_else(|| format!("{}@apple.qent.local", apple_sub));
                let full_name = body
                    .full_name
                    .clone()
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| "Apple User".to_string());
                let id = Uuid::new_v4();
                let now = Utc::now().naive_utc();

                let insert = sqlx::query(
                    r#"INSERT INTO users (id, email, password_hash, full_name, role, verification_status, wallet_balance, is_active, country, apple_id, created_at, updated_at)
                       VALUES ($1, $2, NULL, $3, $4, $5, 0.0, true, $6, $7, $8, $8)"#,
                )
                .bind(id)
                .bind(&email)
                .bind(&full_name)
                .bind(crate::models::UserRole::Renter)
                .bind(VerificationStatus::Pending)
                .bind("Nigeria")
                .bind(&apple_sub)
                .bind(now)
                .execute(pool.get_ref())
                .await;

                if let Err(e) = insert {
                    log::error!("Failed to create Apple user: {e}");
                    return HttpResponse::InternalServerError()
                        .json(serde_json::json!({"error": "Failed to create account"}));
                }

                match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
                    .bind(id)
                    .fetch_one(pool.get_ref())
                    .await
                {
                    Ok(u) => u,
                    Err(e) => {
                        log::error!("Failed to load newly created Apple user: {e}");
                        return HttpResponse::InternalServerError()
                            .json(serde_json::json!({"error": "Failed to create account"}));
                    }
                }
            }
        }
        Err(e) => {
            log::error!("DB error looking up Apple user: {e}");
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Internal server error"}));
        }
    };

    // Issue our JWT + rotate refresh token (same shape as /signin)
    let jwt_claims = Claims {
        sub: user.id,
        role: user.role.clone(),
        exp: (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = match sign_token(&jwt_claims, &config.jwt_secret) {
        Ok(t) => t,
        Err(resp) => return resp,
    };

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

/// POST /api/auth/signin/google
///
/// Accepts a Google OpenID Connect `idToken`, verifies it against Google's JWKS,
/// and signs the user into Qent. Existing users are matched by `google_id` first,
/// then linked by verified email, otherwise a new renter account is created.
#[utoipa::path(
    post,
    path = "/api/auth/signin/google",
    tag = "Auth",
    request_body = GoogleSignInRequest,
    responses(
        (status = 200, description = "Authenticated via Google", body = AuthResponseWithRefresh),
        (status = 401, description = "Invalid Google ID token"),
    ),
)]
pub async fn sign_in_with_google(
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    body: web::Json<GoogleSignInRequest>,
) -> HttpResponse {
    let claims = match verify_google_identity_token(&body.id_token, &config.google_client_ids).await
    {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Google idToken verification failed: {e}");
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Invalid Google identity token"}));
        }
    };

    let google_sub = claims.sub;
    let google_email = claims.email.or_else(|| body.email.clone());
    let email_verified = claims.email_verified.unwrap_or(false);

    let existing =
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE google_id = $1 AND is_active = true")
            .bind(&google_sub)
            .fetch_optional(pool.get_ref())
            .await;

    let user = match existing {
        Ok(Some(u)) => u,
        Ok(None) => {
            let by_email = if email_verified {
                if let Some(ref email) = google_email {
                    sqlx::query_as::<_, User>(
                        "SELECT * FROM users WHERE LOWER(email) = LOWER($1) AND is_active = true",
                    )
                    .bind(email)
                    .fetch_optional(pool.get_ref())
                    .await
                    .ok()
                    .flatten()
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(u) = by_email {
                let _ = sqlx::query(
                    "UPDATE users SET google_id = $1, updated_at = NOW() WHERE id = $2",
                )
                .bind(&google_sub)
                .bind(u.id)
                .execute(pool.get_ref())
                .await;
                User {
                    google_id: Some(google_sub.clone()),
                    ..u
                }
            } else {
                let email = google_email
                    .clone()
                    .unwrap_or_else(|| format!("{}@google.qent.local", google_sub));
                let full_name = body
                    .full_name
                    .clone()
                    .or(claims.name)
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| "Google User".to_string());
                let id = Uuid::new_v4();
                let now = Utc::now().naive_utc();

                let insert = sqlx::query(
                    r#"INSERT INTO users (id, email, password_hash, full_name, role, verification_status, wallet_balance, is_active, country, google_id, created_at, updated_at)
                       VALUES ($1, $2, NULL, $3, $4, $5, 0.0, true, $6, $7, $8, $8)"#,
                )
                .bind(id)
                .bind(&email)
                .bind(&full_name)
                .bind(crate::models::UserRole::Renter)
                .bind(VerificationStatus::Pending)
                .bind("Nigeria")
                .bind(&google_sub)
                .bind(now)
                .execute(pool.get_ref())
                .await;

                if let Err(e) = insert {
                    log::error!("Failed to create Google user: {e}");
                    if e.to_string().contains("users_email_key")
                        || e.to_string().contains("duplicate key")
                    {
                        return HttpResponse::Conflict().json(serde_json::json!({
                            "error": "Email already registered. Sign in with your existing method first, then link Google."
                        }));
                    }
                    return HttpResponse::InternalServerError()
                        .json(serde_json::json!({"error": "Failed to create account"}));
                }

                match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
                    .bind(id)
                    .fetch_one(pool.get_ref())
                    .await
                {
                    Ok(u) => u,
                    Err(e) => {
                        log::error!("Failed to load newly created Google user: {e}");
                        return HttpResponse::InternalServerError()
                            .json(serde_json::json!({"error": "Failed to create account"}));
                    }
                }
            }
        }
        Err(e) => {
            log::error!("DB error looking up Google user: {e}");
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Internal server error"}));
        }
    };

    let jwt_claims = Claims {
        sub: user.id,
        role: user.role.clone(),
        exp: (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = match sign_token(&jwt_claims, &config.jwt_secret) {
        Ok(t) => t,
        Err(resp) => return resp,
    };

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
