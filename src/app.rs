use actix::Addr;
use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{middleware::Logger, web, App};

use crate::auth_mw;
use crate::handlers;
use crate::openapi::ApiDoc;
use crate::services::push::PushService;
use crate::services::sumsub::SumsubClient;
use crate::services::wallet::WalletClient;
use crate::services::AppConfig;
use sqlx::PgPool;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn build_app(
    pool: PgPool,
    config: AppConfig,
    ws_manager: Addr<handlers::ws::WsManager>,
    push_service: Option<PushService>,
    wallet_client: WalletClient,
    sumsub_client: SumsubClient,
) -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let cors = Cors::default()
        .allowed_origin("http://localhost:3000")
        .allowed_origin("http://127.0.0.1:3000")
        .allowed_origin("http://localhost:8080")
        .allowed_origin("http://10.0.2.2:8080")
        .allowed_origin("https://qent.online")
        .allowed_origin("https://www.qent.online")
        .allowed_origin("https://qent.netlify.app")
        .allowed_origin("https://thriving-bonbon-08b8ce.netlify.app")
        .allowed_origin("http://qent.online")
        .allowed_origin("http://www.qent.online")
        .allowed_origin("http://localhost:5173")
        .allowed_origin("http://localhost:5174")
        .allowed_origin("http://localhost:5175")
        .allowed_origin("http://localhost:5176")
        .allowed_origin("http://127.0.0.1:5500")
        .allowed_origin("http://localhost:5500")
        .allowed_origin("http://127.0.0.1:5501")
        .allowed_origin("http://localhost:5501")
        .allow_any_method()
        .allow_any_header()
        .max_age(3600);

    let auth_rate_limit = GovernorConfigBuilder::default()
        .seconds_per_request(6)
        .burst_size(10)
        .finish()
        .unwrap();

    App::new()
        .wrap(cors)
        .wrap(Logger::default())
        .app_data(web::Data::new(pool.clone()))
        .app_data(web::Data::new(config.clone()))
        .app_data(web::Data::new(ws_manager.clone()))
        .app_data(web::Data::new(push_service.clone()))
        .app_data(web::Data::new(wallet_client.clone()))
        .app_data(web::Data::new(sumsub_client.clone()))
        .route("/health", web::get().to(handlers::health::health_check))
        .route("/ws", web::get().to(handlers::ws::ws_connect))
        .service(actix_files::Files::new("/uploads", "uploads").show_files_listing())
        .service(
            SwaggerUi::new("/api/docs/{_:.*}").url("/api/docs/openapi.json", ApiDoc::openapi()),
        )
        .service(
            web::scope("/api")
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
                        )
                        .route(
                            "/privy",
                            web::post().to(handlers::auth_privy::exchange_privy),
                        ),
                )
                .route(
                    "/countries",
                    web::get().to(handlers::countries::list_countries),
                )
                .route("/cars/search", web::get().to(handlers::cars_search::search))
                .route(
                    "/cars/homepage",
                    web::get().to(handlers::cars::get_homepage),
                )
                .route(
                    "/cars/{id}/view",
                    web::post().to(handlers::dashboard::increment_view),
                )
                .route("/cars/{id}", web::get().to(handlers::cars::get_car))
                .route(
                    "/protection-plans",
                    web::get().to(handlers::protection_plans::list_plans),
                )
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
                .route(
                    "/payments/banks",
                    web::get().to(handlers::payments::list_banks),
                )
                .route(
                    "/payments/verify-account",
                    web::post().to(handlers::payments::verify_bank_account),
                )
                .route(
                    "/waitlist",
                    web::post().to(handlers::waitlist::join_waitlist),
                )
                .route(
                    "/waitlist/count",
                    web::get().to(handlers::waitlist::waitlist_count),
                )
                .route(
                    "/payments/webhook",
                    web::post().to(handlers::payments::paystack_webhook),
                )
                .service(
                    web::resource("/webhooks/alchemy/usdc-receive")
                        .app_data(web::PayloadConfig::new(10 * 1024 * 1024))
                        .route(web::post().to(handlers::webhook_chain::usdc_receive)),
                )
                .service(
                    web::resource("/webhooks/sumsub")
                        .app_data(web::PayloadConfig::new(10 * 1024 * 1024))
                        .route(web::post().to(handlers::webhook_sumsub::decision)),
                )
                .configure(configure_authenticated_routes),
        )
}

fn configure_authenticated_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .wrap(actix_web::middleware::from_fn(auth_mw))
            .route("/profile", web::get().to(handlers::auth::get_profile))
            .route("/profile", web::put().to(handlers::auth::update_profile))
            .route(
                "/profile/verify-identity",
                web::post().to(handlers::auth::verify_identity),
            )
            .route(
                "/users/me/country",
                web::post().to(handlers::countries::set_my_country),
            )
            .route(
                "/kyc/access-token",
                web::post().to(handlers::kyc::access_token),
            )
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
            .route(
                "/dashboard/stats",
                web::get().to(handlers::dashboard::get_host_stats),
            )
            .route(
                "/dashboard/listings",
                web::get().to(handlers::dashboard::get_host_listings),
            )
            .route(
                "/bookings",
                web::post().to(handlers::bookings::create_booking),
            )
            .route(
                "/bookings/{id}/pay-usdc",
                web::post().to(handlers::bookings_pay::request_payment_intent),
            )
            .route(
                "/payments/{id}/submit-tx",
                web::post().to(handlers::bookings_pay::submit_tx),
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
                    .route("/earnings", web::get().to(handlers::payments::get_earnings))
                    .route(
                        "/initiate",
                        web::post().to(handlers::payments::initiate_payment),
                    )
                    .route("/withdraw", web::post().to(handlers::payments::withdraw))
                    .route(
                        "/refund/{id}",
                        web::post().to(handlers::payments::request_refund),
                    )
                    .route(
                        "/verify",
                        web::post().to(handlers::payments::verify_payment),
                    ),
            )
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
            .route("/reviews", web::post().to(handlers::reviews::create_review))
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
            .route(
                "/devices/register",
                web::post().to(handlers::devices::register_device_token),
            )
            .route(
                "/devices/{token}",
                web::delete().to(handlers::devices::unregister_device_token),
            )
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
            .route(
                "/partner/profile",
                web::get().to(handlers::partner_onboarding::get_profile),
            )
            .route(
                "/partner/profile",
                web::post().to(handlers::partner_onboarding::upsert_profile),
            )
            .route(
                "/partner/listings",
                web::get().to(handlers::partner_onboarding::list_listings),
            )
            .route(
                "/partner/listings",
                web::post().to(handlers::partner_onboarding::create_listing),
            )
            .route(
                "/partner/listings/draft",
                web::get().to(handlers::partner_onboarding::get_draft_listing),
            )
            .route(
                "/partner/listings/{id}/photos",
                web::put().to(handlers::partner_onboarding::update_listing_photos),
            )
            .route(
                "/partner/listings/{id}/docs",
                web::post().to(handlers::partner_onboarding::submit_listing_docs),
            )
            .route(
                "/partner/listings/{id}/owner-consent",
                web::post().to(handlers::partner_onboarding::submit_owner_consent),
            )
            .route(
                "/partner/listings/{id}/pricing",
                web::post().to(handlers::partner_onboarding::set_listing_pricing),
            )
            .route(
                "/partner/listings/{id}/submit",
                web::post().to(handlers::partner_onboarding::submit_listing),
            )
            .route(
                "/partner/email/mark-verified",
                web::post().to(handlers::partner_onboarding::mark_email_verified),
            )
            .route("/stories", web::get().to(handlers::stories::get_stories))
            .route("/stories", web::post().to(handlers::stories::create_story))
            .route(
                "/stories/{id}",
                web::delete().to(handlers::stories::delete_story),
            )
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
            .route(
                "/turn-credentials",
                web::get().to(handlers::turn::get_turn_credentials),
            )
            .route("/upload", web::post().to(handlers::upload::upload_file))
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
            .route(
                "/damage-reports",
                web::post().to(handlers::damage_reports::create_report),
            )
            .route(
                "/damage-reports/{id}",
                web::get().to(handlers::damage_reports::get_reports),
            )
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
                "/admin/partner-listings",
                web::get().to(handlers::admin::list_partner_listings),
            )
            .route(
                "/admin/partner-listings/{id}/approve",
                web::post().to(handlers::admin::approve_partner_listing),
            )
            .route(
                "/admin/partner-listings/{id}/reject",
                web::post().to(handlers::admin::reject_partner_listing),
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
            )
            .route(
                "/admin/waitlist",
                web::get().to(handlers::waitlist::admin_list_waitlist),
            ),
    );
}
