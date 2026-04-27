use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use hmac::{Hmac, Mac};
use sha2::Sha512;
use sqlx::PgPool;
use uuid::Uuid;

use crate::services::push::PushService;
use crate::models::{
    Booking, BookingStatus, Claims, EarningEntry, EarningsStats, InitiatePaymentRequest, Payment,
    PaymentInitResponse, PaymentStatus, PaystackWebhookEvent, TransactionType,
    VerifyAccountRequest, WalletTransaction,
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
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
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
        .header(
            "Authorization",
            format!("Bearer {}", config.paystack_secret_key),
        )
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
    push: web::Data<Option<PushService>>,
    body_bytes: web::Bytes,
) -> HttpResponse {
    // Verify Paystack signature (HMAC SHA-512)
    let signature = match req.headers().get("x-paystack-signature") {
        Some(sig) => sig.to_str().unwrap_or("").to_string(),
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Missing signature"}))
        }
    };

    let mut mac = Hmac::<Sha512>::new_from_slice(config.paystack_secret_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(&body_bytes);
    let expected = hex::encode(mac.finalize().into_bytes());

    if signature != expected {
        return HttpResponse::Unauthorized()
            .json(serde_json::json!({"error": "Invalid signature"}));
    }

    let body: PaystackWebhookEvent = match serde_json::from_slice(&body_bytes) {
        Ok(b) => b,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid payload"}))
        }
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
            let _ =
                sqlx::query("UPDATE bookings SET status = $1, updated_at = NOW() WHERE id = $2")
                    .bind(BookingStatus::Confirmed)
                    .bind(payment.booking_id)
                    .execute(pool.get_ref())
                    .await;

            // Notify the host about the new confirmed booking
            if let Ok(Some(booking)) =
                sqlx::query_as::<_, Booking>("SELECT * FROM bookings WHERE id = $1")
                    .bind(payment.booking_id)
                    .fetch_optional(pool.get_ref())
                    .await
            {
                // Get renter name for the notification
                let renter_name =
                    sqlx::query_scalar::<_, String>("SELECT full_name FROM users WHERE id = $1")
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

                let push_title = "New Booking Confirmed!".to_string();
                let push_body = format!("{} has booked your {} — payment confirmed.", renter_name, car_name);
                let push_data = serde_json::json!({
                    "booking_id": booking.id,
                    "car_id": booking.car_id,
                    "renter_id": payment.payer_id,
                });

                let _ = sqlx::query(
                    r#"INSERT INTO notifications (id, user_id, title, message, notification_type, is_read, data, created_at)
                    VALUES ($1, $2, $3, $4, $5, false, $6, NOW())"#,
                )
                .bind(Uuid::new_v4())
                .bind(booking.host_id)
                .bind(&push_title)
                .bind(&push_body)
                .bind("booking_confirmed")
                .bind(push_data.clone())
                .execute(pool.get_ref())
                .await;

                if let Some(push) = push.get_ref().clone() {
                    let pool = pool.get_ref().clone();
                    let host_id = booking.host_id;
                    tokio::spawn(async move {
                        push.send_to_user(&pool, host_id, &push_title, &push_body, push_data).await;
                    });
                }

                // Send booking confirmation email to the renter
                let renter_email =
                    sqlx::query_scalar::<_, String>("SELECT email FROM users WHERE id = $1")
                        .bind(payment.payer_id)
                        .fetch_one(pool.get_ref())
                        .await
                        .unwrap_or_default();

                if !renter_email.is_empty() {
                    let email_service = EmailService::new(config.resend_api_key.clone());
                    email_service
                        .send_booking_confirmation(
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
                        )
                        .await;
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
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let balance = sqlx::query_scalar::<_, f64>("SELECT wallet_balance FROM users WHERE id = $1")
        .bind(claims.sub)
        .fetch_one(pool.get_ref())
        .await;

    match balance {
        Ok(b) => HttpResponse::Ok().json(serde_json::json!({"balance": b})),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// POST /api/payments/verify — Verify a payment with Paystack after user returns from browser.
/// This is the fallback for when the webhook can't reach the server (e.g., local dev).
pub async fn verify_payment(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    push: web::Data<Option<PushService>>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let reference = match body["reference"].as_str() {
        Some(r) => r.to_string(),
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "Missing reference"}))
        }
    };

    // Check if payment exists and belongs to this user
    let payment = sqlx::query_as::<_, Payment>(
        "SELECT * FROM payments WHERE provider_reference = $1 AND payer_id = $2",
    )
    .bind(&reference)
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await;

    let payment = match payment {
        Ok(Some(p)) => p,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({"error": "Payment not found"}))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };

    // Already confirmed?
    if payment.status == PaymentStatus::Success {
        return HttpResponse::Ok()
            .json(serde_json::json!({"status": "success", "message": "Payment already verified"}));
    }

    // Verify with Paystack
    let client = reqwest::Client::new();
    let verify_resp = client
        .get(format!(
            "https://api.paystack.co/transaction/verify/{}",
            reference
        ))
        .header(
            "Authorization",
            format!("Bearer {}", config.paystack_secret_key),
        )
        .send()
        .await;

    let paystack_data = match verify_resp {
        Ok(resp) => resp.json::<serde_json::Value>().await.unwrap_or_default(),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": format!("Paystack error: {}", e)}))
        }
    };

    let paystack_status = paystack_data["data"]["status"].as_str().unwrap_or("");

    if paystack_status != "success" {
        return HttpResponse::Ok().json(serde_json::json!({
            "status": "pending",
            "message": format!("Payment status: {}", paystack_status),
        }));
    }

    // Payment confirmed — update payment status
    let _ = sqlx::query("UPDATE payments SET status = $1 WHERE id = $2")
        .bind(PaymentStatus::Success)
        .bind(payment.id)
        .execute(pool.get_ref())
        .await;

    // Update booking to confirmed
    let _ = sqlx::query("UPDATE bookings SET status = $1, updated_at = NOW() WHERE id = $2")
        .bind(BookingStatus::Confirmed)
        .bind(payment.booking_id)
        .execute(pool.get_ref())
        .await;

    // Get booking for notification
    if let Ok(Some(booking)) = sqlx::query_as::<_, Booking>("SELECT * FROM bookings WHERE id = $1")
        .bind(payment.booking_id)
        .fetch_optional(pool.get_ref())
        .await
    {
        let renter_name =
            sqlx::query_scalar::<_, String>("SELECT full_name FROM users WHERE id = $1")
                .bind(claims.sub)
                .fetch_one(pool.get_ref())
                .await
                .unwrap_or_else(|_| "Someone".to_string());

        let car_name = sqlx::query_scalar::<_, String>(
            "SELECT CONCAT(make, ' ', model, ' ', year) FROM cars WHERE id = $1",
        )
        .bind(booking.car_id)
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or_else(|_| "your car".to_string());

        // Notify host
        let push_title = "Payment Confirmed!".to_string();
        let push_body = format!("{} has paid for your {} — ready for pickup.", renter_name, car_name);
        let push_data = serde_json::json!({
            "booking_id": booking.id,
            "car_id": booking.car_id,
            "renter_id": claims.sub,
        });

        let _ = sqlx::query(
            r#"INSERT INTO notifications (id, user_id, title, message, notification_type, is_read, data, created_at)
            VALUES ($1, $2, $3, $4, $5, false, $6, NOW())"#,
        )
        .bind(Uuid::new_v4())
        .bind(booking.host_id)
        .bind(&push_title)
        .bind(&push_body)
        .bind("booking_confirmed")
        .bind(push_data.clone())
        .execute(pool.get_ref())
        .await;

        if let Some(push) = push.get_ref().clone() {
            let pool = pool.get_ref().clone();
            let host_id = booking.host_id;
            tokio::spawn(async move {
                push.send_to_user(&pool, host_id, &push_title, &push_body, push_data).await;
            });
        }
    }

    HttpResponse::Ok().json(serde_json::json!({"status": "success", "message": "Payment verified and booking confirmed"}))
}

pub async fn get_wallet_transactions(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let transactions = sqlx::query_as::<_, WalletTransaction>(
        "SELECT * FROM wallet_transactions WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(claims.sub)
    .fetch_all(pool.get_ref())
    .await;

    match transactions {
        Ok(t) => HttpResponse::Ok().json(t),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// POST /api/payments/withdraw — Host withdraws wallet balance via Paystack transfer
pub async fn withdraw(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    body: web::Json<crate::models::PayoutRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    if body.amount < 1000.0 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Minimum withdrawal is ₦1,000"}));
    }

    // Check balance
    let balance = sqlx::query_scalar::<_, f64>("SELECT wallet_balance FROM users WHERE id = $1")
        .bind(claims.sub)
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0.0);

    if body.amount > balance {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Insufficient wallet balance", "balance": balance}));
    }

    // Large withdrawals (> ₦100,000) require admin approval
    if body.amount > 100_000.0 {
        // Debit wallet immediately (hold funds)
        let _ = sqlx::query(
            "UPDATE users SET wallet_balance = wallet_balance - $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(body.amount)
        .bind(claims.sub)
        .execute(pool.get_ref())
        .await;

        // Record as pending withdrawal
        let _ = sqlx::query(
            r#"INSERT INTO wallet_transactions (id, user_id, amount, balance_after, description, status, created_at)
            VALUES ($1, $2, $3, (SELECT wallet_balance FROM users WHERE id = $2), $4, 'pending_approval', NOW())"#,
        )
        .bind(Uuid::new_v4())
        .bind(claims.sub)
        .bind(-body.amount)
        .bind(format!(
            "Withdrawal to ****{} (pending approval)",
            &body.account_number[body.account_number.len().saturating_sub(4)..]
        ))
        .execute(pool.get_ref())
        .await;

        return HttpResponse::Ok().json(serde_json::json!({
            "message": "Withdrawal request submitted for approval",
            "amount": body.amount,
            "status": "pending_approval",
            "note": "Withdrawals over ₦100,000 require admin approval"
        }));
    }

    // Step 1: Create a Paystack transfer recipient
    let client = reqwest::Client::new();
    let recipient_resp = client
        .post("https://api.paystack.co/transferrecipient")
        .header(
            "Authorization",
            format!("Bearer {}", config.paystack_secret_key),
        )
        .json(&serde_json::json!({
            "type": "nuban",
            "name": claims.sub.to_string(),
            "account_number": body.account_number,
            "bank_code": body.bank_code,
            "currency": "NGN"
        }))
        .send()
        .await;

    let recipient_code = match recipient_resp {
        Ok(resp) => {
            let json: serde_json::Value = resp.json().await.unwrap_or_default();
            if json["status"].as_bool() != Some(true) {
                let msg = json["message"]
                    .as_str()
                    .unwrap_or("Failed to create transfer recipient");
                return HttpResponse::BadRequest().json(serde_json::json!({"error": msg}));
            }
            json["data"]["recipient_code"]
                .as_str()
                .unwrap_or("")
                .to_string()
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": format!("Payment provider error: {}", e)}));
        }
    };

    // Step 2: Initiate transfer
    let reference = format!("qent_wd_{}", Uuid::new_v4());
    let amount_kobo = (body.amount * 100.0) as i64;

    let transfer_resp = client
        .post("https://api.paystack.co/transfer")
        .header(
            "Authorization",
            format!("Bearer {}", config.paystack_secret_key),
        )
        .json(&serde_json::json!({
            "source": "balance",
            "reason": "Qent host withdrawal",
            "amount": amount_kobo,
            "recipient": recipient_code,
            "reference": reference
        }))
        .send()
        .await;

    match transfer_resp {
        Ok(resp) => {
            let json: serde_json::Value = resp.json().await.unwrap_or_default();
            if json["status"].as_bool() != Some(true) {
                let msg = json["message"].as_str().unwrap_or("Transfer failed");
                return HttpResponse::BadRequest().json(serde_json::json!({"error": msg}));
            }

            // Debit wallet
            let _ = sqlx::query(
                "UPDATE users SET wallet_balance = wallet_balance - $1, updated_at = NOW() WHERE id = $2",
            )
            .bind(body.amount)
            .bind(claims.sub)
            .execute(pool.get_ref())
            .await;

            // Record wallet transaction (negative amount = debit)
            let _ = sqlx::query(
                r#"INSERT INTO wallet_transactions (id, user_id, amount, balance_after, description, created_at)
                VALUES ($1, $2, $3, (SELECT wallet_balance FROM users WHERE id = $2), $4, NOW())"#,
            )
            .bind(Uuid::new_v4())
            .bind(claims.sub)
            .bind(-body.amount)
            .bind(format!("Withdrawal to bank account ****{}", &body.account_number[body.account_number.len().saturating_sub(4)..]))
            .execute(pool.get_ref())
            .await;

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Withdrawal initiated",
                "amount": body.amount,
                "reference": reference,
                "status": "processing"
            }))
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": format!("Transfer failed: {}", e)})),
    }
}

/// GET /api/payments/earnings — Host earnings breakdown
pub async fn get_earnings(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    // Total earnings, this month, pending (approved/active bookings)
    let stats = sqlx::query_as::<_, EarningsStats>(
        r#"SELECT
            COALESCE(SUM(CASE WHEN status = 'completed' THEN subtotal * 0.85 ELSE 0 END), 0) as total_earned,
            COALESCE(SUM(CASE WHEN status = 'completed' AND updated_at >= date_trunc('month', NOW()) THEN subtotal * 0.85 ELSE 0 END), 0) as this_month,
            COALESCE(SUM(CASE WHEN status IN ('approved', 'confirmed', 'active') THEN subtotal * 0.85 ELSE 0 END), 0) as pending_earnings,
            COUNT(CASE WHEN status = 'completed' THEN 1 END)::int as completed_trips
        FROM bookings WHERE host_id = $1"#,
    )
    .bind(claims.sub)
    .fetch_one(pool.get_ref())
    .await;

    let balance = sqlx::query_scalar::<_, f64>("SELECT wallet_balance FROM users WHERE id = $1")
        .bind(claims.sub)
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0.0);

    // Recent completed bookings with earnings
    let recent = sqlx::query_as::<_, EarningEntry>(
        r#"SELECT b.id as booking_id, (c.make || ' ' || c.model) as car_name,
            b.subtotal * 0.85 as earned, b.updated_at as completed_at,
            u.full_name as renter_name
        FROM bookings b
        LEFT JOIN cars c ON c.id = b.car_id
        LEFT JOIN users u ON u.id = b.renter_id
        WHERE b.host_id = $1 AND b.status = 'completed'
        ORDER BY b.updated_at DESC LIMIT 20"#,
    )
    .bind(claims.sub)
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    match stats {
        Ok(s) => HttpResponse::Ok().json(serde_json::json!({
            "total_earned": s.total_earned,
            "this_month": s.this_month,
            "pending_earnings": s.pending_earnings,
            "completed_trips": s.completed_trips,
            "wallet_balance": balance,
            "platform_fee_percent": 15,
            "recent_earnings": recent
        })),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// GET /api/payments/banks — List Nigerian banks (via Paystack + logos)
pub async fn list_banks(config: web::Data<AppConfig>) -> HttpResponse {
    let client = reqwest::Client::new();

    // Fetch banks from Paystack
    let paystack_resp = client
        .get("https://api.paystack.co/bank?country=nigeria")
        .header(
            "Authorization",
            format!("Bearer {}", config.paystack_secret_key),
        )
        .send()
        .await;

    let banks = match paystack_resp {
        Ok(r) => {
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            json["data"].as_array().cloned().unwrap_or_default()
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": format!("Failed to fetch banks: {}", e)}));
        }
    };

    // Fetch logos from nigerianbanks.xyz (best-effort)
    let logos_resp = client.get("https://nigerianbanks.xyz").send().await;

    let logo_map: std::collections::HashMap<String, String> = match logos_resp {
        Ok(r) => {
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            json.as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|b| {
                            let name = b["name"].as_str()?.to_lowercase();
                            let logo = b["logo"].as_str()?.to_string();
                            Some((name, logo))
                        })
                        .collect()
                })
                .unwrap_or_default()
        }
        Err(_) => std::collections::HashMap::new(),
    };

    // Merge logos into bank data
    let enriched: Vec<serde_json::Value> = banks
        .into_iter()
        .map(|mut bank| {
            let name = bank["name"].as_str().unwrap_or("").to_lowercase();
            // Try to find a matching logo by fuzzy name match
            let logo = logo_map.iter().find_map(|(logo_name, url)| {
                if name.contains(logo_name) || logo_name.contains(&name) {
                    Some(url.clone())
                } else {
                    // Try matching first word
                    let first_word = name.split_whitespace().next().unwrap_or("");
                    if first_word.len() > 3 && logo_name.contains(first_word) {
                        Some(url.clone())
                    } else {
                        None
                    }
                }
            });
            if let Some(logo_url) = logo {
                bank["logo"] = serde_json::Value::String(logo_url);
            }
            bank
        })
        .collect();

    HttpResponse::Ok().json(enriched)
}

/// POST /api/payments/verify-account — Verify bank account details
pub async fn verify_bank_account(
    config: web::Data<AppConfig>,
    body: web::Json<VerifyAccountRequest>,
) -> HttpResponse {
    let client = reqwest::Client::new();
    let resp = client
        .get(&format!(
            "https://api.paystack.co/bank/resolve?account_number={}&bank_code={}",
            body.account_number, body.bank_code
        ))
        .header(
            "Authorization",
            format!("Bearer {}", config.paystack_secret_key),
        )
        .send()
        .await;

    match resp {
        Ok(r) => {
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            if json["status"].as_bool() == Some(true) {
                HttpResponse::Ok().json(serde_json::json!({
                    "account_name": json["data"]["account_name"],
                    "account_number": json["data"]["account_number"],
                    "bank_code": body.bank_code
                }))
            } else {
                let msg = json["message"]
                    .as_str()
                    .unwrap_or("Could not resolve account");
                HttpResponse::BadRequest().json(serde_json::json!({"error": msg}))
            }
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": format!("Verification failed: {}", e)})),
    }
}

pub async fn request_refund(
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
