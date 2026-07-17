use actix::Actor;
use actix_web::body::MessageBody;
use actix_web::dev::ServiceResponse;
use actix_web::middleware::Next;
use actix_web::{dev::ServiceRequest, web, Error, HttpServer};
use sqlx::postgres::PgPoolOptions;

mod app;
mod handlers;
mod jobs;
mod middleware;
mod models;
mod openapi;
mod services;

use crate::middleware::auth::validate_token;
use crate::services::push::PushService;
use crate::services::sumsub::SumsubClient;
use crate::services::wallet::WalletClient;
use crate::services::AppConfig;

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
    tokio::spawn(jobs::auto_complete_bookings::run(bg_pool));

    // Start WebSocket connection manager
    let ws_manager = handlers::ws::WsManager::new().start();

    // V2 §4.3 — fallback path for missed Alchemy webhooks. Idle if
    // ALCHEMY_RPC_URL / ESCROW_WALLET_ADDRESS aren't set.
    jobs::reconcile_chain_payments::spawn(pool.clone(), config.clone(), ws_manager.clone());

    // V2 clients — built once and shared. Both tolerate missing env
    // (the underlying methods return NotConfigured).
    let wallet_client = WalletClient::new(
        config.privy_app_id.clone(),
        config.privy_app_secret.clone(),
        config.privy_jwks_url.clone(),
    );
    let sumsub_client = SumsubClient::new(
        config.sumsub_app_token.clone(),
        config.sumsub_secret_key.clone(),
        config.sumsub_webhook_secret.clone(),
    );

    HttpServer::new(move || {
        app::build_app(
            pool.clone(),
            config.clone(),
            ws_manager.clone(),
            push_service.clone(),
            wallet_client.clone(),
            sumsub_client.clone(),
        )
    })
    .bind(&bind_addr)?
    .run()
    .await
}
