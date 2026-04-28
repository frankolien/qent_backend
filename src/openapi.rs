//! OpenAPI 3.0 spec — generated from `#[utoipa::path]` annotations.
//!
//! Add new endpoints by:
//! 1. Annotating the handler with `#[utoipa::path(...)]`
//! 2. Listing the handler in `paths(...)` below
//! 3. Listing every request/response struct in `components(schemas(...))` below
//!
//! Then visit `/api/docs` to see Swagger UI populated with the new route.

use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::handlers;
use crate::models::{
    AppleSignInRequest, AuthResponseWithRefresh, Car, CarStatus, CreateCarRequest,
    ForgotPasswordRequest, GoogleSignInRequest, RefreshTokenRequest, ResetPasswordRequest,
    SignInRequest, SignUpRequest, UpdateCarRequest, UpdateProfileRequest, UserPublic, UserRole,
    VerificationStatus, VerifyIdentityRequest,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Qent API",
        version = "1.0.0",
        description = "REST API powering the Qent peer-to-peer car rental platform.\n\n\
                       **Authentication**: most endpoints require a JWT access token in the \
                       `Authorization: Bearer <token>` header. Get one from `/api/auth/signin` \
                       or `/api/auth/signup`.",
        contact(name = "Qent", url = "https://qent.online"),
    ),
    servers(
        (url = "https://qent-backend.onrender.com", description = "Production"),
        (url = "http://localhost:8080", description = "Local development"),
    ),
    paths(
        // Auth
        handlers::auth::sign_up,
        handlers::auth::sign_in,
        handlers::auth::refresh_token,
        handlers::auth::forgot_password,
        handlers::auth::reset_password,
        handlers::auth::get_profile,
        handlers::auth::update_profile,
        handlers::auth::verify_identity,
        handlers::auth::sign_in_with_apple,
        handlers::auth::sign_in_with_google,
        // Users (public)
        handlers::auth::get_user_public,
        // Cars
        handlers::cars::search_cars,
        handlers::cars::get_homepage,
        handlers::cars::get_car,
        handlers::cars::create_car,
        handlers::cars::get_host_cars,
        handlers::cars::update_car,
        handlers::cars::deactivate_car,
        handlers::cars::get_booked_dates,
        // Favorites
        handlers::favorites::toggle_favorite,
        handlers::favorites::get_favorites,
        handlers::favorites::check_favorite,
    ),
    components(schemas(
        // Auth request bodies
        SignUpRequest,
        SignInRequest,
        RefreshTokenRequest,
        ForgotPasswordRequest,
        ResetPasswordRequest,
        UpdateProfileRequest,
        VerifyIdentityRequest,
        AppleSignInRequest,
        GoogleSignInRequest,
        // Auth responses
        AuthResponseWithRefresh,
        UserPublic,
        UserRole,
        VerificationStatus,
        // Cars
        Car,
        CarStatus,
        CreateCarRequest,
        UpdateCarRequest,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "Auth", description = "Sign up, sign in, password recovery, profile management"),
        (name = "Users", description = "Public user profiles"),
        (name = "Cars", description = "Car listings: search, browse, host CRUD"),
        (name = "Favorites", description = "User saved cars"),
    ),
)]
pub struct ApiDoc;

/// Registers the `bearer_auth` security scheme so handlers tagged with
/// `security(("bearer_auth" = []))` show a lock icon + "Authorize" button
/// in Swagger UI.
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi
            .components
            .as_mut()
            .expect("ApiDoc has components defined");
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("Paste a JWT obtained from /api/auth/signin"))
                    .build(),
            ),
        );
    }
}
