use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use hmac::{Hmac, Mac};
use sha2::Sha512;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    Booking, BookingStatus, Claims, InitiatePaymentRequest, Payment, PaymentInitResponse,
    PaymentStatus, PaystackWebhookEvent, TransactionType, WalletTransaction,
};
use crate::services::email::EmailService;
use crate::services::AppConfig;

pub async fn initiate_payment(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    body: web::Json<InitiatePaymentRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    let booking = match sqlx::query_as::<_, Booking>(
        "SELECT * FROM bookings WHERE id = $1 AND renter_id = $2 AND status IN ('pending', 'approved')",
    )
    .bind(body.booking_id)
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(b)) => b,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": "Booking not found or not approved"}))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };

    let reference = format!("qent_{}", Uuid::new_v4());
    let amount_kobo = (booking.total_amount * 100.0) as i64;

    // Get user email for Paystack
    let email = sqlx::query_scalar::<_, String>("SELECT email FROM users WHERE id = $1")
        .bind(claims.sub)
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or_default();

    // Initialize Paystack transaction
    let client = reqwest::Client::new();
    let paystack_resp = client
        .post("https://api.paystack.co/transaction/initialize")
        .header("Authorization", format!("Bearer {}", config.paystack_secret_key))
        .json(&serde_json::json!({
            "email": email,
            "amount": amount_kobo,
            "reference": reference,
            "currency": "NGN",
            "callback_url": format!("{}/api/payments/callback", config.app_url),
        }))
        .send()
        .await;

    let authorization_url = match paystack_resp {
        Ok(resp) => {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            body["data"]["authorization_url"]
                .as_str()
                .unwrap_or("")
                .to_string()
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": format!("Payment provider error: {}", e)}));
        }
    };

    // Record payment
    let _ = sqlx::query(
        r#"INSERT INTO payments (id, booking_id, payer_id, amount, currency, provider, provider_reference, status, transaction_type, created_at)
        VALUES ($1, $2, $3, $4, 'NGN', 'paystack', $5, $6, $7, NOW())"#,
    )
    .bind(Uuid::new_v4())
    .bind(booking.id)
    .bind(claims.sub)
    .bind(booking.total_amount)
    .bind(&reference)
    .bind(PaymentStatus::Pending)
    .bind(TransactionType::Payment)
    .execute(pool.get_ref())
    .await;

    HttpResponse::Ok().json(PaymentInitResponse {
        authorization_url,
        reference,
    })
}

pub async fn paystack_webhook(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    body_bytes: web::Bytes,
) -> HttpResponse {
    // Verify Paystack signature (HMAC SHA-512)
    let signature = match req.headers().get("x-paystack-signature") {
        Some(sig) => sig.to_str().unwrap_or("").to_string(),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Missing signature"})),
    };

    let mut mac = Hmac::<Sha512>::new_from_slice(config.paystack_secret_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(&body_bytes);
    let expected = hex::encode(mac.finalize().into_bytes());

    if signature != expected {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid signature"}));
    }

    let body: PaystackWebhookEvent = match serde_json::from_slice(&body_bytes) {
        Ok(b) => b,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid payload"})),
    };

    if body.event == "charge.success" {
        let reference = &body.data.reference;

        // Update payment status
        let payment = sqlx::query_as::<_, Payment>(
            "UPDATE payments SET status = $1 WHERE provider_reference = $2 RETURNING *",
        )
        .bind(PaymentStatus::Success)
        .bind(reference)
        .fetch_optional(pool.get_ref())
        .await;

        if let Ok(Some(payment)) = payment {
            // Update booking to confirmed
            let _ = sqlx::query(
                "UPDATE bookings SET status = $1, updated_at = NOW() WHERE id = $2",
            )
            .bind(BookingStatus::Confirmed)
            .bind(payment.booking_id)
            .execute(pool.get_ref())
            .await;

            // Notify the host about the new confirmed booking
            if let Ok(Some(booking)) = sqlx::query_as::<_, Booking>(
                "SELECT * FROM bookings WHERE id = $1",
            )
            .bind(payment.booking_id)
            .fetch_optional(pool.get_ref())
            .await
            {
                // Get renter name for the notification
                let renter_name = sqlx::query_scalar::<_, String>(
                    "SELECT full_name FROM users WHERE id = $1",
                )
                .bind(payment.payer_id)
                .fetch_one(pool.get_ref())
                .await
                .unwrap_or_else(|_| "Someone".to_string());

                // Get car name
                let car_name = sqlx::query_scalar::<_, String>(
                    "SELECT CONCAT(make, ' ', model, ' ', year) FROM cars WHERE id = $1",
                )
                .bind(booking.car_id)
                .fetch_one(pool.get_ref())
                .await
                .unwrap_or_else(|_| "your car".to_string());

                let _ = sqlx::query(
                    r#"INSERT INTO notifications (id, user_id, title, message, notification_type, is_read, data, created_at)
                    VALUES ($1, $2, $3, $4, $5, false, $6, NOW())"#,
                )
                .bind(Uuid::new_v4())
                .bind(booking.host_id)
                .bind("New Booking Confirmed!")
                .bind(format!("{} has booked your {} — payment confirmed.", renter_name, car_name))
                .bind("booking_confirmed")
                .bind(serde_json::json!({
                    "booking_id": booking.id,
                    "car_id": booking.car_id,
                    "renter_id": payment.payer_id,
                }))
                .execute(pool.get_ref())
                .await;

                // Send booking confirmation email to the renter
                let renter_email = sqlx::query_scalar::<_, String>(
                    "SELECT email FROM users WHERE id = $1",
                )
                .bind(payment.payer_id)
                .fetch_one(pool.get_ref())
                .await
                .unwrap_or_default();

                if !renter_email.is_empty() {
                    let email_service = EmailService::new(config.resend_api_key.clone());
                    email_service.send_booking_confirmation(
                        &renter_email,
                        &renter_name,
                        &car_name,
                        &booking.id.to_string(),
                        booking.start_date,
                        booking.end_date,
                        booking.total_days,
                        booking.subtotal,
                        booking.service_fee,
                        booking.protection_fee,
                        booking.total_amount,
                        reference,
                    ).await;
                }
            }

            // Auto-save card if authorization is reusable
            if let Some(auth) = &body.data.authorization {
                if auth.reusable == Some(true) {
                    let last4 = auth.last4.clone().unwrap_or_default();
                    // Check if card already saved (by last4 + exp for this user)
                    let already_saved = sqlx::query_scalar::<_, bool>(
                        "SELECT EXISTS(SELECT 1 FROM saved_cards WHERE user_id = $1 AND last4 = $2 AND authorization_code = $3)",
                    )
                    .bind(payment.payer_id)
                    .bind(&last4)
                    .bind(&auth.authorization_code)
                    .fetch_one(pool.get_ref())
                    .await
                    .unwrap_or(false);

                    if !already_saved {
                        // Check if user has any cards (first card becomes default)
                        let has_cards = sqlx::query_scalar::<_, bool>(
                            "SELECT EXISTS(SELECT 1 FROM saved_cards WHERE user_id = $1)",
                        )
                        .bind(payment.payer_id)
                        .fetch_one(pool.get_ref())
                        .await
                        .unwrap_or(false);

                        let _ = sqlx::query(
                            r#"INSERT INTO saved_cards (id, user_id, authorization_code, card_type, last4, exp_month, exp_year, bin, bank, brand, is_default, cardholder_name, created_at)
                            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW())"#,
                        )
                        .bind(Uuid::new_v4())
                        .bind(payment.payer_id)
                        .bind(&auth.authorization_code)
                        .bind(auth.card_type.as_deref().unwrap_or("unknown"))
                        .bind(&last4)
                        .bind(auth.exp_month.as_deref().unwrap_or(""))
                        .bind(auth.exp_year.as_deref().unwrap_or(""))
                        .bind(auth.bin.as_deref().unwrap_or(""))
                        .bind(&auth.bank)
                        .bind(auth.brand.as_deref().unwrap_or("Unknown"))
                        .bind(!has_cards) // First card is default
                        .bind(&auth.account_name)
                        .execute(pool.get_ref())
                        .await;
                    }
                }
            }
        }
    }

    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

pub async fn get_wallet_balance(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    let balance = sqlx::query_scalar::<_, f64>("SELECT wallet_balance FROM users WHERE id = $1")
        .bind(claims.sub)
        .fetch_one(pool.get_ref())
        .await;

    match balance {
        Ok(b) => HttpResponse::Ok().json(serde_json::json!({"balance": b})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn get_wallet_transactions(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    let transactions = sqlx::query_as::<_, WalletTransaction>(
        "SELECT * FROM wallet_transactions WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(claims.sub)
    .fetch_all(pool.get_ref())
    .await;

    match transactions {
        Ok(t) => HttpResponse::Ok().json(t),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn request_refund(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})),
    };

    let booking_id = path.into_inner();

    let booking = match sqlx::query_as::<_, Booking>(
        "SELECT * FROM bookings WHERE id = $1 AND renter_id = $2 AND status = 'cancelled'",
    )
    .bind(booking_id)
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(b)) => b,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": "No cancelled booking found"}))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };

    // Check if already refunded
    let already_refunded = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM payments WHERE booking_id = $1 AND transaction_type = 'refund')",
    )
    .bind(booking_id)
    .fetch_one(pool.get_ref())
    .await;

    if let Ok(true) = already_refunded {
        return HttpResponse::Conflict().json(serde_json::json!({"error": "Already refunded"}));
    }

    let refund_amount = booking.total_amount * 0.90; // 90% refund (10% cancellation fee)

    let _ = sqlx::query(
        r#"INSERT INTO payments (id, booking_id, payer_id, amount, currency, provider, status, transaction_type, created_at)
        VALUES ($1, $2, $3, $4, 'NGN', 'paystack', $5, $6, NOW())"#,
    )
    .bind(Uuid::new_v4())
    .bind(booking_id)
    .bind(claims.sub)
    .bind(refund_amount)
    .bind(PaymentStatus::Pending)
    .bind(TransactionType::Refund)
    .execute(pool.get_ref())
    .await;

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Refund initiated",
        "refund_amount": refund_amount
    }))
}
