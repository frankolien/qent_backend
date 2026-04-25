use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Renter,
    Host,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "verification_status", rename_all = "lowercase")]
pub enum VerificationStatus {
    Pending,
    Verified,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub phone: Option<String>,
    pub password_hash: Option<String>,
    pub full_name: String,
    pub role: UserRole,
    pub profile_photo_url: Option<String>,
    pub drivers_license_url: Option<String>,
    pub id_card_url: Option<String>,
    pub verification_status: VerificationStatus,
    pub wallet_balance: f64,
    pub is_active: bool,
    pub country: String,
    pub apple_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SignUpRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
    #[validate(length(min = 2))]
    pub full_name: String,
    pub phone: Option<String>,
    pub role: UserRole,
    pub country: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserPublic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPublic {
    pub id: Uuid,
    pub email: String,
    pub phone: Option<String>,
    pub full_name: String,
    pub role: UserRole,
    pub profile_photo_url: Option<String>,
    pub verification_status: VerificationStatus,
    pub wallet_balance: f64,
    pub is_active: bool,
    pub country: String,
    pub created_at: NaiveDateTime,
}

impl From<User> for UserPublic {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            phone: u.phone,
            full_name: u.full_name,
            role: u.role,
            profile_photo_url: u.profile_photo_url,
            verification_status: u.verification_status,
            wallet_balance: u.wallet_balance,
            is_active: u.is_active,
            country: u.country,
            created_at: u.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct VerifyIdentityRequest {
    pub drivers_license_url: String,
    pub id_card_url: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 2))]
    pub full_name: Option<String>,
    pub phone: Option<String>,
    pub profile_photo_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub role: UserRole,
    pub exp: usize,
}

#[derive(Debug, Serialize)]
pub struct AuthResponseWithRefresh {
    pub token: String,
    pub refresh_token: String,
    pub user: UserPublic,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct AppleSignInRequest {
    pub identity_token: String,
    pub full_name: Option<String>,
    pub email: Option<String>,
}
