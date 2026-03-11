use actix_cors::Cors;
use actix_web::{dev::ServiceRequest, middleware::Logger, web, App, Error, HttpServer};
use actix_web::body::MessageBody;
use actix_web::dev::ServiceResponse;
use actix_web::middleware::Next;
use sqlx::postgres::PgPoolOptions;

mod handlers;
mod middleware;
mod models;
mod services;

use crate::middleware::auth::validate_token;
use crate::services::AppConfig;

async fn auth_mw(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let jwt_secret = req
        .app_data::<web::Data<AppConfig>>()
        .map(|c| c.jwt_secret.clone())
        .unwrap_or_default();
    validate_token(&req, &jwt_secret)?;
    next.call(req).await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let config = AppConfig::from_env();
    let bind_addr = format!("{}:{}", config.host, config.port);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("Failed to create database pool");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    log::info!("Qent API starting on {}", bind_addr);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://127.0.0.1:3000")
            .allowed_origin("http://localhost:8080")
            .allowed_origin("http://10.0.2.2:8080") // Android emulator
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .service(
                web::scope("/api")
                    // Auth - public
                    .route("/auth/signup", web::post().to(handlers::auth::sign_up))
                    .route("/auth/signin", web::post().to(handlers::auth::sign_in))
                    // Cars - public search
                    .route("/cars/search", web::get().to(handlers::cars::search_cars))
                    .route("/cars/{id}", web::get().to(handlers::cars::get_car))
                    // Protection plans - public
                    .route("/protection-plans", web::get().to(handlers::protection_plans::list_plans))
                    // Reviews - public
                    .route("/users/{id}/reviews", web::get().to(handlers::reviews::get_user_reviews))
                    .route("/users/{id}/rating", web::get().to(handlers::reviews::get_user_rating))
                    // Email verification - no auth (pre-signup)
                    .route("/auth/send-code", web::post().to(handlers::verification::send_code))
                    .route("/auth/verify-code", web::post().to(handlers::verification::verify_code))
                    // Paystack webhook - no auth
                    .route("/payments/webhook", web::post().to(handlers::payments::paystack_webhook))
                    // Authenticated routes
                    .service(
                        web::scope("")
                            .wrap(actix_web::middleware::from_fn(auth_mw))
                            // Profile
                            .route("/auth/profile", web::get().to(handlers::auth::get_profile))
                            .route("/auth/profile", web::put().to(handlers::auth::update_profile))
                            .route("/auth/verify-identity", web::post().to(handlers::auth::verify_identity))
                            // Cars - host
                            .route("/cars", web::post().to(handlers::cars::create_car))
                            .route("/cars/my-listings", web::get().to(handlers::cars::get_host_cars))
                            .route("/cars/{id}", web::put().to(handlers::cars::update_car))
                            .route("/cars/{id}/deactivate", web::post().to(handlers::cars::deactivate_car))
                            // Bookings
                            .route("/bookings", web::post().to(handlers::bookings::create_booking))
                            .route("/bookings/mine", web::get().to(handlers::bookings::get_my_bookings))
                            .route("/bookings/{id}", web::get().to(handlers::bookings::get_booking))
                            .route("/bookings/{id}/action", web::post().to(handlers::bookings::update_booking_status))
                            // Payments
                            .route("/payments/initiate", web::post().to(handlers::payments::initiate_payment))
                            .route("/payments/wallet", web::get().to(handlers::payments::get_wallet_balance))
                            .route("/payments/wallet/transactions", web::get().to(handlers::payments::get_wallet_transactions))
                            .route("/payments/refund/{id}", web::post().to(handlers::payments::request_refund))
                            // Saved Cards
                            .route("/cards", web::get().to(handlers::cards::list_cards))
                            .route("/cards/{id}/default", web::post().to(handlers::cards::set_default_card))
                            .route("/cards/{id}", web::delete().to(handlers::cards::delete_card))
                            .route("/cards/charge", web::post().to(handlers::cards::charge_saved_card))
                            // Reviews
                            .route("/reviews", web::post().to(handlers::reviews::create_review))
                            // Favorites
                            .route("/favorites", web::get().to(handlers::favorites::get_favorites))
                            .route("/favorites/{id}", web::post().to(handlers::favorites::toggle_favorite))
                            .route("/favorites/{id}/check", web::get().to(handlers::favorites::check_favorite))
                            // Notifications
                            .route("/notifications", web::get().to(handlers::notifications::get_notifications))
                            .route("/notifications/{id}/read", web::post().to(handlers::notifications::mark_read))
                            .route("/notifications/read-all", web::post().to(handlers::notifications::mark_all_read))
                            // Partnership
                            .route("/partner/apply", web::post().to(handlers::partner::apply))
                            .route("/partner/application", web::get().to(handlers::partner::get_application))
                            .route("/partner/dashboard", web::get().to(handlers::partner::dashboard))
                            .route("/partner/activate-car", web::post().to(handlers::partner::activate_car))
                            // Admin
                            .route("/admin/users", web::get().to(handlers::admin::list_users))
                            .route("/admin/users/{id}/verify", web::post().to(handlers::admin::verify_user))
                            .route("/admin/users/{id}/reject", web::post().to(handlers::admin::reject_user_verification))
                            .route("/admin/users/{id}/deactivate", web::post().to(handlers::admin::deactivate_user))
                            .route("/admin/cars", web::get().to(handlers::admin::list_all_cars))
                            .route("/admin/cars/{id}/approve", web::post().to(handlers::admin::approve_car))
                            .route("/admin/cars/{id}/reject", web::post().to(handlers::admin::reject_car))
                            .route("/admin/bookings", web::get().to(handlers::admin::list_all_bookings))
                            .route("/admin/bookings/{id}/dispute-refund", web::post().to(handlers::admin::handle_dispute_refund))
                            .route("/admin/payments", web::get().to(handlers::admin::list_all_payments))
                            .route("/admin/analytics", web::get().to(handlers::admin::get_analytics))
                    ),
            )
    })
    .bind(&bind_addr)?
    .run()
    .await
}
