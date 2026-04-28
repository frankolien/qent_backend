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
    AppleSignInRequest, AuthResponseWithRefresh, Booking, BookingAction, BookingActionRequest,
    BookingStatus, BookingWithCar, Car, CarStatus, CreateBookingRequest, CreateCarRequest,
    EarningEntry, EarningsStats, ForgotPasswordRequest, GoogleSignInRequest,
    InitiatePaymentRequest, Notification, Payment, PaymentInitResponse, PaymentStatus,
    PayoutRequest, RefreshTokenRequest, RegisterDeviceTokenRequest, ResetPasswordRequest,
    SavedCardPublic, SignInRequest, SignUpRequest, TransactionType, UpdateCarRequest,
    UpdateProfileRequest, UserPublic, UserRole, VerificationStatus, VerifyAccountRequest,
    VerifyIdentityRequest, WalletTransaction,
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
        // Bookings
        handlers::bookings::create_booking,
        handlers::bookings::get_booking,
        handlers::bookings::get_my_bookings,
        handlers::bookings::update_booking_status,
        handlers::bookings::get_host_pending_bookings,
        // Payments
        handlers::payments::initiate_payment,
        handlers::payments::paystack_webhook,
        handlers::payments::verify_payment,
        handlers::payments::get_wallet_balance,
        handlers::payments::get_wallet_transactions,
        handlers::payments::withdraw,
        handlers::payments::get_earnings,
        handlers::payments::list_banks,
        handlers::payments::verify_bank_account,
        handlers::payments::request_refund,
        // Cards
        handlers::cards::list_cards,
        handlers::cards::set_default_card,
        handlers::cards::delete_card,
        handlers::cards::charge_saved_card,
        // Chat
        handlers::chat::get_or_create_conversation,
        handlers::chat::get_conversations,
        handlers::chat::get_messages,
        handlers::chat::send_message,
        handlers::chat::delete_conversation,
        handlers::chat::mark_read,
        // Notifications
        handlers::notifications::get_notifications,
        handlers::notifications::mark_read,
        handlers::notifications::mark_all_read,
        handlers::notifications::delete_notification,
        handlers::notifications::delete_bulk,
        // Devices
        handlers::devices::register_device_token,
        handlers::devices::unregister_device_token,
        // Stories
        handlers::stories::get_stories,
        handlers::stories::create_story,
        handlers::stories::delete_story,
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
        // Bookings
        Booking,
        BookingWithCar,
        BookingStatus,
        BookingAction,
        BookingActionRequest,
        CreateBookingRequest,
        // Payments
        Payment,
        PaymentStatus,
        TransactionType,
        InitiatePaymentRequest,
        PaymentInitResponse,
        PayoutRequest,
        VerifyAccountRequest,
        WalletTransaction,
        EarningsStats,
        EarningEntry,
        // Cards
        SavedCardPublic,
        crate::handlers::cards::ChargeSavedCardRequest,
        // Chat
        crate::handlers::chat::ConversationResponse,
        crate::handlers::chat::MessageResponse,
        crate::handlers::chat::CreateConversationRequest,
        crate::handlers::chat::SendMessageRequest,
        // Notifications
        Notification,
        crate::handlers::notifications::BulkDeleteRequest,
        // Devices
        RegisterDeviceTokenRequest,
        // Stories
        crate::handlers::stories::StoryResponse,
        crate::handlers::stories::CreateStoryRequest,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "Auth", description = "Sign up, sign in, password recovery, profile management"),
        (name = "Users", description = "Public user profiles"),
        (name = "Cars", description = "Car listings: search, browse, host CRUD"),
        (name = "Favorites", description = "User saved cars"),
        (name = "Bookings", description = "Booking lifecycle: create, approve/reject, activate, complete"),
        (name = "Payments", description = "Paystack payments, wallet, withdrawals, banks, refunds"),
        (name = "Cards", description = "Saved card management for recurring charges"),
        (name = "Chat", description = "1:1 conversations between renter and host with realtime messages"),
        (name = "Notifications", description = "In-app notification feed (FCM push is delivered separately)"),
        (name = "Devices", description = "Push notification token registration"),
        (name = "Stories", description = "Host-posted 24-hour expiring stories"),
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
