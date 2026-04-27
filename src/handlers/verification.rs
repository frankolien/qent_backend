use actix_web::{web, HttpResponse};
use chrono::{Duration, Utc};
use rand::Rng;
use sqlx::PgPool;

use crate::services::AppConfig;

#[derive(Debug, serde::Deserialize)]
pub struct SendCodeRequest {
    pub email: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct VerifyCodeRequest {
    pub email: String,
    pub code: String,
}

/// Generate a 4-digit verification code
fn generate_code() -> String {
    let mut rng = rand::thread_rng();
    let code: u16 = rng.gen_range(1000..=9999);
    code.to_string()
}

/// Send verification code to email via Resend, store in DB
pub async fn send_code(
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    body: web::Json<SendCodeRequest>,
) -> HttpResponse {
    let email = body.email.trim().to_lowercase();

    if !email.contains('@') || !email.contains('.') {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Invalid email address"}));
    }

    let code = generate_code();
    let expires_at = Utc::now() + Duration::minutes(5);

    // Delete any existing codes for this email
    let _ = sqlx::query("DELETE FROM verification_codes WHERE email = $1")
        .bind(&email)
        .execute(pool.get_ref())
        .await;

    // Insert new code
    let result =
        sqlx::query("INSERT INTO verification_codes (email, code, expires_at) VALUES ($1, $2, $3)")
            .bind(&email)
            .bind(&code)
            .bind(expires_at)
            .execute(pool.get_ref())
            .await;

    if result.is_err() {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Failed to generate verification code"}));
    }

    // Send email via Resend
    let sent = send_resend_email(&config.resend_api_key, &email, &code).await;

    if sent {
        // Clean up expired codes periodically
        let _ = sqlx::query("DELETE FROM verification_codes WHERE expires_at < NOW()")
            .execute(pool.get_ref())
            .await;

        HttpResponse::Ok().json(serde_json::json!({
            "message": "Verification code sent",
            "expires_in_seconds": 300
        }))
    } else {
        // Remove the code we just stored since email failed
        let _ = sqlx::query("DELETE FROM verification_codes WHERE email = $1")
            .bind(&email)
            .execute(pool.get_ref())
            .await;

        HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Failed to send verification email"}))
    }
}

/// Verify the code entered by the user
pub async fn verify_code(
    pool: web::Data<PgPool>,
    body: web::Json<VerifyCodeRequest>,
) -> HttpResponse {
    let email = body.email.trim().to_lowercase();

    let row = sqlx::query_as::<_, VerificationRow>(
        "SELECT code, verified, expires_at FROM verification_codes WHERE email = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&email)
    .fetch_optional(pool.get_ref())
    .await;

    let row = match row {
        Ok(Some(r)) => r,
        _ => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "No verification code found. Please request a new one."}));
        }
    };

    if row.verified {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Code already used"}));
    }

    if Utc::now() > row.expires_at {
        // Clean up expired code
        let _ = sqlx::query("DELETE FROM verification_codes WHERE email = $1")
            .bind(&email)
            .execute(pool.get_ref())
            .await;
        return HttpResponse::BadRequest().json(
            serde_json::json!({"error": "Verification code expired. Please request a new one."}),
        );
    }

    if row.code != body.code.trim() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Invalid verification code"}));
    }

    // Mark as verified
    let _ = sqlx::query("UPDATE verification_codes SET verified = true WHERE email = $1")
        .bind(&email)
        .execute(pool.get_ref())
        .await;

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Email verified successfully",
        "verified": true
    }))
}

#[derive(Debug, sqlx::FromRow)]
struct VerificationRow {
    code: String,
    verified: bool,
    expires_at: chrono::DateTime<Utc>,
}

/// Send email via Resend API
async fn send_resend_email(api_key: &str, email: &str, code: &str) -> bool {
    if api_key.is_empty() {
        log::warn!("RESEND_API_KEY not set, skipping email send");
        return false;
    }

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.resend.com/emails")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "from": "Qent <noreply@qent.online>",
            "to": email,
            "subject": "Your Qent Verification Code",
            "html": build_email_html(code)
        }))
        .send()
        .await;

    match resp {
        Ok(r) => {
            let status = r.status();
            if status.is_success() {
                log::info!("Verification email sent to {}", email);
                true
            } else {
                let body = r.text().await.unwrap_or_default();
                log::error!("Resend API error ({}): {}", status, body);
                false
            }
        }
        Err(e) => {
            log::error!("Failed to call Resend API: {}", e);
            false
        }
    }
}

fn build_email_html(code: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <style>
    body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px; background-color: #f5f5f5; }}
    .container {{ background-color: #fff; border-radius: 12px; padding: 40px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
    .logo {{ text-align: center; margin-bottom: 30px; }}
    .logo h1 {{ color: #000; font-size: 28px; font-weight: bold; margin: 0; }}
    .code-container {{ background-color: #f8f8f8; border-radius: 8px; padding: 20px; text-align: center; margin: 30px 0; }}
    .code {{ font-size: 32px; font-weight: bold; color: #000; letter-spacing: 8px; font-family: 'Courier New', monospace; }}
    .footer {{ text-align: center; margin-top: 30px; color: #666; font-size: 12px; }}
  </style>
</head>
<body>
  <div class="container">
    <div class="logo"><h1>Qent</h1></div>
    <h2 style="color: #000; font-size: 24px; margin-bottom: 20px;">Enter verification code</h2>
    <p style="color: #666; font-size: 14px;">We sent a code to your email address. Enter it below to verify your account.</p>
    <div class="code-container"><div class="code">{code}</div></div>
    <p style="color: #666; font-size: 14px;">This code expires in 5 minutes.</p>
    <p style="color: #666; font-size: 14px;">If you didn't request this, please ignore this email.</p>
    <div class="footer"><p>&copy; 2024 Qent. All rights reserved.</p></div>
  </div>
</body>
</html>"#
    )
}
