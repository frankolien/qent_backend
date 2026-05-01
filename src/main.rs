use actix::Actor;
use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::body::MessageBody;
use actix_web::dev::ServiceResponse;
use actix_web::middleware::Next;
use actix_web::{dev::ServiceRequest, middleware::Logger, web, App, Error, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

mod handlers;
mod middleware;
mod models;
mod openapi;
mod services;

use crate::middleware::auth::validate_token;
use crate::openapi::ApiDoc;
use crate::services::push::PushService;
use crate::services::AppConfig;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Background task: auto-complete bookings past their end_date (runs every hour)
async fn auto_complete_bookings(pool: PgPool) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
    loop {
        interval.tick().await;
        log::info!("Running auto-complete check for overdue bookings...");

        // Find active bookings past their end_date
        let overdue = sqlx::query_as::<_, (uuid::Uuid, uuid::Uuid, f64)>(
            r#"SELECT id, host_id, subtotal FROM bookings
               WHERE status = 'active' AND end_date < CURRENT_DATE"#,
        )
        .fetch_all(&pool)
        .await;

        match overdue {
            Ok(bookings) => {
                for (booking_id, host_id, subtotal) in &bookings {
                    // Mark as completed
                    let _ = sqlx::query(
                        "UPDATE bookings SET status = 'completed', updated_at = NOW() WHERE id = $1",
                    )
                    .bind(booking_id)
                    .execute(&pool)
                    .await;

                    // Credit host wallet (85%)
                    let payout = subtotal * 0.85;
                    let _ = sqlx::query(
                        "UPDATE users SET wallet_balance = wallet_balance + $1, updated_at = NOW() WHERE id = $2",
                    )
                    .bind(payout)
                    .bind(host_id)
                    .execute(&pool)
                    .await;

                    // Record wallet transaction
                    let _ = sqlx::query(
                        r#"INSERT INTO wallet_transactions (id, user_id, amount, balance_after, description, reference_id, created_at)
                        VALUES ($1, $2, $3, (SELECT wallet_balance FROM users WHERE id = $2), $4, $5, NOW())"#,
                    )
                    .bind(uuid::Uuid::new_v4())
                    .bind(host_id)
                    .bind(payout)
                    .bind(format!("Auto-payout for booking {}", booking_id))
                    .bind(booking_id)
                    .execute(&pool)
                    .await;

                    // Notify renter
                    let _ = sqlx::query(
                        r#"INSERT INTO notifications (id, user_id, title, message, notification_type, is_read, data, created_at)
                        VALUES ($1, (SELECT renter_id FROM bookings WHERE id = $2), 'Trip Completed',
                        'Your trip has been auto-completed. Leave a review!', 'booking_completed', false, $3, NOW())"#,
                    )
                    .bind(uuid::Uuid::new_v4())
                    .bind(booking_id)
                    .bind(serde_json::json!({"booking_id": booking_id.to_string()}))
                    .execute(&pool)
                    .await;
                }
                if !bookings.is_empty() {
                    log::info!("Auto-completed {} overdue booking(s)", bookings.len());
                }
            }
            Err(e) => log::error!("Auto-complete query failed: {}", e),
        }
    }
}

async fn auth_mw<B>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<actix_web::body::EitherBody<B>>, Error>
where
    B: MessageBody + 'static,
{
    let jwt_secret = req
        .app_data::<web::Data<AppConfig>>()
        .map(|c| c.jwt_secret.clone())
        .unwrap_or_default();
    // Don't propagate the auth failure as an actix `Error` — that
    // short-circuits the chain and skips outer middlewares (notably
    // actix-cors), so the 401 reaches the browser without an
    // Access-Control-Allow-Origin header and the browser blocks it.
    // Symptom in the admin dashboard: every request fails with
    // "blocked by CORS policy". Build a real response instead so it
    // flows back through cors and picks up the headers.
    if let Err(e) = validate_token(&req, &jwt_secret) {
        let body = serde_json::json!({"error": e.to_string()}).to_string();
        let resp = actix_web::HttpResponse::Unauthorized()
            .insert_header(("content-type", "application/json"))
            .body(body);
        return Ok(req.into_response(resp).map_into_right_body());
    }
    next.call(req).await.map(|r| r.map_into_left_body())
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

    // Initialize push notification service (optional — pushes are no-op if it fails)
    let push_service = match PushService::from_env() {
        Ok(svc) => {
            log::info!("PushService initialized");
            Some(svc)
        }
        Err(e) => {
            log::warn!("PushService disabled: {}", e);
            None
        }
    };

    // Spawn background auto-completion task
    let bg_pool = pool.clone();
    tokio::spawn(auto_complete_bookings(bg_pool));

    // Rate limiter configs
    // Auth: 10 requests per minute per IP
    let auth_rate_limit = GovernorConfigBuilder::default()
        .seconds_per_request(6)
        .burst_size(10)
        .finish()
        .unwrap();

    // Payments: 5 requests per minute per IP
    let payment_rate_limit = GovernorConfigBuilder::default()
        .seconds_per_request(12)
        .burst_size(5)
        .finish()
        .unwrap();

    // Start WebSocket connection manager
    let ws_manager = handlers::ws::WsManager::new().start();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://127.0.0.1:3000")
            .allowed_origin("http://localhost:8080")
            .allowed_origin("http://10.0.2.2:8080") // Android emulator
            .allowed_origin("https://qent.online")
            .allowed_origin("https://www.qent.online")
            .allowed_origin("https://qent.netlify.app") // Netlify default domain
            .allowed_origin("https://thriving-bonbon-08b8ce.netlify.app")
            .allowed_origin("http://qent.online")
            .allowed_origin("http://www.qent.online")
            .allowed_origin("http://localhost:5173") // Vite dev server
            .allowed_origin("http://localhost:5174")
            .allowed_origin("http://localhost:5175")
            .allowed_origin("http://localhost:5176")
            .allowed_origin("http://127.0.0.1:5500") // VS Code Live Server
            .allowed_origin("http://localhost:5500")
            .allowed_origin("http://127.0.0.1:5501")
            .allowed_origin("http://localhost:5501")
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(ws_manager.clone()))
            .app_data(web::Data::new(push_service.clone()))
            .route("/health", web::get().to(handlers::health::health_check))
            .route("/ws", web::get().to(handlers::ws::ws_connect))
            .service(actix_files::Files::new("/uploads", "uploads").show_files_listing())
            // OpenAPI / Swagger UI — Swagger UI at /api/docs/, raw spec at
            // /api/docs/openapi.json. Mounted before auth scope so it's public.
            .service(
                SwaggerUi::new("/api/docs/{_:.*}")
                    .url("/api/docs/openapi.json", ApiDoc::openapi()),
            )
            .service(
                web::scope("/api")
                    // Auth - public (rate limited)
                    .service(
                        web::scope("/auth")
                            .wrap(Governor::new(&auth_rate_limit))
                            .route("/signup", web::post().to(handlers::auth::sign_up))
                            .route("/signin", web::post().to(handlers::auth::sign_in))
                            .route(
                                "/signin/apple",
                                web::post().to(handlers::auth::sign_in_with_apple),
                            )
                            .route(
                                "/signin/google",
                                web::post().to(handlers::auth::sign_in_with_google),
                            )
                            .route("/refresh", web::post().to(handlers::auth::refresh_token))
                            .route(
                                "/forgot-password",
                                web::post().to(handlers::auth::forgot_password),
                            )
                            .route(
                                "/reset-password",
                                web::post().to(handlers::auth::reset_password),
                            )
                            .route(
                                "/send-code",
                                web::post().to(handlers::verification::send_code),
                            )
                            .route(
                                "/verify-code",
                                web::post().to(handlers::verification::verify_code),
                            ),
                    )
                    // Cars - public
                    .route("/cars/search", web::get().to(handlers::cars::search_cars))
                    .route(
                        "/cars/homepage",
                        web::get().to(handlers::cars::get_homepage),
                    )
                    .route(
                        "/cars/{id}/view",
                        web::post().to(handlers::dashboard::increment_view),
                    )
                    .route("/cars/{id}", web::get().to(handlers::cars::get_car))
                    // Protection plans - public
                    .route(
                        "/protection-plans",
                        web::get().to(handlers::protection_plans::list_plans),
                    )
                    // Reviews - public
                    .route(
                        "/users/{id}",
                        web::get().to(handlers::auth::get_user_public),
                    )
                    .route(
                        "/users/{id}/reviews",
                        web::get().to(handlers::reviews::get_user_reviews),
                    )
                    .route(
                        "/users/{id}/rating",
                        web::get().to(handlers::reviews::get_user_rating),
                    )
                    .route(
                        "/cars/{id}/reviews",
                        web::get().to(handlers::reviews::get_car_reviews),
                    )
                    // Banks - public (for withdrawal form)
                    .route(
                        "/payments/banks",
                        web::get().to(handlers::payments::list_banks),
                    )
                    .route(
                        "/payments/verify-account",
                        web::post().to(handlers::payments::verify_bank_account),
                    )
                    // Waitlist - public
                    .route(
                        "/waitlist",
                        web::post().to(handlers::waitlist::join_waitlist),
                    )
                    .route(
                        "/waitlist/count",
                        web::get().to(handlers::waitlist::waitlist_count),
                    )
                    // Paystack webhook - no auth
                    .route(
                        "/payments/webhook",
                        web::post().to(handlers::payments::paystack_webhook),
                    )
                    // Authenticated routes
                    .service(
                        web::scope("")
                            .wrap(actix_web::middleware::from_fn(auth_mw))
                            // Profile
                            .route("/profile", web::get().to(handlers::auth::get_profile))
                            .route("/profile", web::put().to(handlers::auth::update_profile))
                            .route(
                                "/profile/verify-identity",
                                web::post().to(handlers::auth::verify_identity),
                            )
                            // Cars - host
                            .route("/cars", web::post().to(handlers::cars::create_car))
                            .route(
                                "/cars/my-listings",
                                web::get().to(handlers::cars::get_host_cars),
                            )
                            .route("/cars/{id}", web::put().to(handlers::cars::update_car))
                            .route(
                                "/cars/{id}/deactivate",
                                web::post().to(handlers::cars::deactivate_car),
                            )
                            .route(
                                "/cars/{id}/booked-dates",
                                web::get().to(handlers::cars::get_booked_dates),
                            )
                            // Host Dashboard
                            .route(
                                "/dashboard/stats",
                                web::get().to(handlers::dashboard::get_host_stats),
                            )
                            .route(
                                "/dashboard/listings",
                                web::get().to(handlers::dashboard::get_host_listings),
                            )
                            // Bookings
                            .route(
                                "/bookings",
                                web::post().to(handlers::bookings::create_booking),
                            )
                            .route(
                                "/bookings/mine",
                                web::get().to(handlers::bookings::get_my_bookings),
                            )
                            .route(
                                "/bookings/{id}",
                                web::get().to(handlers::bookings::get_booking),
                            )
                            .route(
                                "/bookings/{id}/action",
                                web::post().to(handlers::bookings::update_booking_status),
                            )
                            .route(
                                "/bookings/host/pending",
                                web::get().to(handlers::bookings::get_host_pending_bookings),
                            )
                            // Payments (write ops are rate-limited, reads are not)
                            .service(
                                web::scope("/payments")
                                    .route(
                                        "/wallet",
                                        web::get().to(handlers::payments::get_wallet_balance),
                                    )
                                    .route(
                                        "/wallet/transactions",
                                        web::get().to(handlers::payments::get_wallet_transactions),
                                    )
                                    .route(
                                        "/earnings",
                                        web::get().to(handlers::payments::get_earnings),
                                    )
                                    .route(
                                        "/initiate",
                                        web::post().to(handlers::payments::initiate_payment),
                                    )
                                    .route(
                                        "/withdraw",
                                        web::post().to(handlers::payments::withdraw),
                                    )
                                    .route(
                                        "/refund/{id}",
                                        web::post().to(handlers::payments::request_refund),
                                    )
                                    .route(
                                        "/verify",
                                        web::post().to(handlers::payments::verify_payment),
                                    ),
                            )
                            // Saved Cards
                            .route("/cards", web::get().to(handlers::cards::list_cards))
                            .route(
                                "/cards/{id}/default",
                                web::post().to(handlers::cards::set_default_card),
                            )
                            .route(
                                "/cards/{id}",
                                web::delete().to(handlers::cards::delete_card),
                            )
                            .route(
                                "/cards/charge",
                                web::post().to(handlers::cards::charge_saved_card),
                            )
                            // Reviews
                            .route("/reviews", web::post().to(handlers::reviews::create_review))
                            // Favorites
                            .route(
                                "/favorites",
                                web::get().to(handlers::favorites::get_favorites),
                            )
                            .route(
                                "/favorites/{id}",
                                web::post().to(handlers::favorites::toggle_favorite),
                            )
                            .route(
                                "/favorites/{id}/check",
                                web::get().to(handlers::favorites::check_favorite),
                            )
                            // Notifications
                            .route(
                                "/notifications",
                                web::get().to(handlers::notifications::get_notifications),
                            )
                            .route(
                                "/notifications/{id}/read",
                                web::post().to(handlers::notifications::mark_read),
                            )
                            .route(
                                "/notifications/read-all",
                                web::post().to(handlers::notifications::mark_all_read),
                            )
                            .route(
                                "/notifications/delete-bulk",
                                web::post().to(handlers::notifications::delete_bulk),
                            )
                            .route(
                                "/notifications/{id}",
                                web::delete().to(handlers::notifications::delete_notification),
                            )
                            // Device tokens (push notifications)
                            .route(
                                "/devices/register",
                                web::post().to(handlers::devices::register_device_token),
                            )
                            .route(
                                "/devices/{token}",
                                web::delete().to(handlers::devices::unregister_device_token),
                            )
                            // Partnership
                            .route("/partner/apply", web::post().to(handlers::partner::apply))
                            .route(
                                "/partner/application",
                                web::get().to(handlers::partner::get_application),
                            )
                            .route(
                                "/partner/dashboard",
                                web::get().to(handlers::partner::dashboard),
                            )
                            .route(
                                "/partner/activate-car",
                                web::post().to(handlers::partner::activate_car),
                            )
                            // Stories
                            .route("/stories", web::get().to(handlers::stories::get_stories))
                            .route("/stories", web::post().to(handlers::stories::create_story))
                            .route(
                                "/stories/{id}",
                                web::delete().to(handlers::stories::delete_story),
                            )
                            // Chat
                            .route(
                                "/chat/conversations",
                                web::post().to(handlers::chat::get_or_create_conversation),
                            )
                            .route(
                                "/chat/conversations",
                                web::get().to(handlers::chat::get_conversations),
                            )
                            .route(
                                "/chat/conversations/{id}/messages",
                                web::get().to(handlers::chat::get_messages),
                            )
                            .route(
                                "/chat/conversations/{id}/messages",
                                web::post().to(handlers::chat::send_message),
                            )
                            .route(
                                "/chat/conversations/{id}/read",
                                web::post().to(handlers::chat::mark_read),
                            )
                            .route(
                                "/chat/conversations/{id}",
                                web::delete().to(handlers::chat::delete_conversation),
                            )
                            // WebRTC: short-lived TURN credentials (Metered)
                            .route(
                                "/turn-credentials",
                                web::get().to(handlers::turn::get_turn_credentials),
                            )
                            // File upload
                            .route("/upload", web::post().to(handlers::upload::upload_file))
                            // Compliance & Account
                            .route(
                                "/auth/accept-terms",
                                web::post().to(handlers::compliance::accept_terms),
                            )
                            .route(
                                "/auth/terms-status",
                                web::get().to(handlers::compliance::terms_status),
                            )
                            .route(
                                "/account/request-deletion",
                                web::post().to(handlers::compliance::request_deletion),
                            )
                            .route(
                                "/account/cancel-deletion",
                                web::post().to(handlers::compliance::cancel_deletion),
                            )
                            .route(
                                "/account/export",
                                web::get().to(handlers::compliance::export_data),
                            )
                            // Damage Reports
                            .route(
                                "/damage-reports",
                                web::post().to(handlers::damage_reports::create_report),
                            )
                            .route(
                                "/damage-reports/{id}",
                                web::get().to(handlers::damage_reports::get_reports),
                            )
                            // Admin
                            .route("/admin/users", web::get().to(handlers::admin::list_users))
                            .route(
                                "/admin/users/{id}/verify",
                                web::post().to(handlers::admin::verify_user),
                            )
                            .route(
                                "/admin/users/{id}/reject",
                                web::post().to(handlers::admin::reject_user_verification),
                            )
                            .route(
                                "/admin/users/{id}/deactivate",
                                web::post().to(handlers::admin::deactivate_user),
                            )
                            .route("/admin/cars", web::get().to(handlers::admin::list_all_cars))
                            .route(
                                "/admin/cars/{id}/approve",
                                web::post().to(handlers::admin::approve_car),
                            )
                            .route(
                                "/admin/cars/{id}/reject",
                                web::post().to(handlers::admin::reject_car),
                            )
                            .route(
                                "/admin/bookings",
                                web::get().to(handlers::admin::list_all_bookings),
                            )
                            .route(
                                "/admin/bookings/{id}/dispute-refund",
                                web::post().to(handlers::admin::handle_dispute_refund),
                            )
                            .route(
                                "/admin/payments",
                                web::get().to(handlers::admin::list_all_payments),
                            )
                            .route(
                                "/admin/analytics",
                                web::get().to(handlers::admin::get_analytics),
                            )
                            .route(
                                "/admin/audit-log",
                                web::get().to(handlers::compliance::admin_audit_log),
                            )
                            .route(
                                "/admin/withdrawals/pending",
                                web::get().to(handlers::admin::list_pending_withdrawals),
                            )
                            .route(
                                "/admin/withdrawals/{id}/approve",
                                web::post().to(handlers::admin::approve_withdrawal),
                            )
                            .route(
                                "/admin/withdrawals/{id}/reject",
                                web::post().to(handlers::admin::reject_withdrawal),
                            ),
                    ),
            )
    })
    .bind(&bind_addr)?
    .run()
    .await
}
