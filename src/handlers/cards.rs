use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::{Claims, SavedCard, SavedCardPublic};
use crate::services::AppConfig;

/// GET /api/cards — List all saved cards for the authenticated user.
#[utoipa::path(
    get,
    path = "/api/cards",
    tag = "Cards",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Saved cards (default first, then by date)", body = Vec<SavedCardPublic>),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn list_cards(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let cards = sqlx::query_as::<_, SavedCard>(
        "SELECT * FROM saved_cards WHERE user_id = $1 ORDER BY is_default DESC, created_at DESC",
    )
    .bind(claims.sub)
    .fetch_all(pool.get_ref())
    .await;

    match cards {
        Ok(cards) => {
            let public: Vec<SavedCardPublic> = cards.into_iter().map(|c| c.into()).collect();
            HttpResponse::Ok().json(public)
        }
        Err(_) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Failed to fetch cards"})),
    }
}

/// POST /api/cards/{id}/default — Set a card as the default payment method.
#[utoipa::path(
    post,
    path = "/api/cards/{id}/default",
    tag = "Cards",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Card ID")),
    responses(
        (status = 200, description = "Default card updated"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Card not found"),
    ),
)]
pub async fn set_default_card(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let card_id = path.into_inner();

    // Verify the card belongs to this user
    let card_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM saved_cards WHERE id = $1 AND user_id = $2)",
    )
    .bind(card_id)
    .bind(claims.sub)
    .fetch_one(pool.get_ref())
    .await;

    if !matches!(card_exists, Ok(true)) {
        return HttpResponse::NotFound().json(serde_json::json!({"error": "Card not found"}));
    }

    // Remove default from all user's cards, then set the new default
    let _ = sqlx::query("UPDATE saved_cards SET is_default = false WHERE user_id = $1")
        .bind(claims.sub)
        .execute(pool.get_ref())
        .await;

    let _ = sqlx::query("UPDATE saved_cards SET is_default = true WHERE id = $1 AND user_id = $2")
        .bind(card_id)
        .bind(claims.sub)
        .execute(pool.get_ref())
        .await;

    HttpResponse::Ok().json(serde_json::json!({"message": "Default card updated"}))
}

/// DELETE /api/cards/{id} — Delete a saved card.
#[utoipa::path(
    delete,
    path = "/api/cards/{id}",
    tag = "Cards",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Card ID")),
    responses(
        (status = 200, description = "Card deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Card not found"),
    ),
)]
pub async fn delete_card(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let card_id = path.into_inner();

    let result = sqlx::query("DELETE FROM saved_cards WHERE id = $1 AND user_id = $2")
        .bind(card_id)
        .bind(claims.sub)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            HttpResponse::Ok().json(serde_json::json!({"message": "Card deleted"}))
        }
        _ => HttpResponse::NotFound().json(serde_json::json!({"error": "Card not found"})),
    }
}

/// POST /api/cards/charge — Charge a saved card for a booking (Paystack recurring authorization).
#[utoipa::path(
    post,
    path = "/api/cards/charge",
    tag = "Cards",
    security(("bearer_auth" = [])),
    request_body = ChargeSavedCardRequest,
    responses(
        (status = 200, description = "Charge successful with Paystack reference"),
        (status = 400, description = "Charge failed at provider"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Card not found"),
        (status = 500, description = "Payment provider error"),
    ),
)]
pub async fn charge_saved_card(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    body: web::Json<ChargeSavedCardRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    // Get the card
    let card =
        sqlx::query_as::<_, SavedCard>("SELECT * FROM saved_cards WHERE id = $1 AND user_id = $2")
            .bind(body.card_id)
            .bind(claims.sub)
            .fetch_optional(pool.get_ref())
            .await;

    let card = match card {
        Ok(Some(c)) => c,
        _ => return HttpResponse::NotFound().json(serde_json::json!({"error": "Card not found"})),
    };

    // Get user email
    let email = sqlx::query_scalar::<_, String>("SELECT email FROM users WHERE id = $1")
        .bind(claims.sub)
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or_default();

    let amount_kobo = (body.amount * 100.0) as i64;

    // Charge via Paystack recurring charge API
    let client = reqwest::Client::new();
    let paystack_resp = client
        .post("https://api.paystack.co/transaction/charge_authorization")
        .header(
            "Authorization",
            format!("Bearer {}", config.paystack_secret_key),
        )
        .json(&serde_json::json!({
            "authorization_code": card.authorization_code,
            "email": email,
            "amount": amount_kobo,
            "currency": "NGN",
        }))
        .send()
        .await;

    match paystack_resp {
        Ok(resp) => {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            if body["status"].as_bool() == Some(true) {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": "Charge successful",
                    "reference": body["data"]["reference"],
                }))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": body["message"].as_str().unwrap_or("Charge failed"),
                }))
            }
        }
        Err(_) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Payment provider error"})),
    }
}

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct ChargeSavedCardRequest {
    pub card_id: Uuid,
    pub amount: f64,
}
