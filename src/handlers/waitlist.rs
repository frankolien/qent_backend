use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::ToSchema;

use crate::services::AppConfig;

async fn send_waitlist_email(api_key: &str, email: &str, name: &str, position: i64, role: &str) {
    if api_key.is_empty() {
        log::warn!("Resend API key not set, skipping waitlist email");
        return;
    }

    let role_line = match role {
        "host" => {
            "You signed up as a <strong>host</strong> — we'll help you start earning with your car."
        }
        "both" => "You signed up as a <strong>renter and host</strong> — the best of both worlds.",
        _ => "You signed up as a <strong>renter</strong> — we'll find the perfect car for you.",
    };

    let early_label = if position <= 50 {
        "You're one of our earliest supporters"
    } else if position <= 200 {
        "You're part of the early wave"
    } else if position <= 500 {
        "You're in great company"
    } else {
        "The movement is growing"
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;margin:0;padding:0;background:#f5f5f5">
<div style="max-width:600px;margin:0 auto;background:#fff">
  <div style="background:#1A1A1A;color:#fff;padding:40px 32px;text-align:center">
    <h1 style="margin:0;font-size:28px;font-weight:800;letter-spacing:-0.5px">You're In</h1>
    <p style="margin:8px 0 0;font-size:14px;color:rgba(255,255,255,0.5)">Welcome to the Qent family</p>
  </div>
  <div style="padding:40px 32px">
    <p style="font-size:18px;font-weight:600;margin-bottom:8px;color:#1A1A1A">Hey{greeting}!</p>
    <p style="color:#666;font-size:15px;line-height:1.7;margin-bottom:24px">
      You've just secured early access to Qent. {early_label} — and we don't take that lightly.
    </p>
    <div style="background:#1A1A1A;border-radius:16px;padding:28px;margin-bottom:24px;text-align:center">
      <p style="margin:0;font-size:14px;color:rgba(255,255,255,0.5);font-weight:500;letter-spacing:1px;text-transform:uppercase">Early Access</p>
      <p style="margin:8px 0 0;font-size:22px;font-weight:800;color:#fff">Secured</p>
    </div>
    <p style="color:#666;font-size:14px;line-height:1.7;margin-bottom:16px">{role_line}</p>
    <p style="color:#666;font-size:14px;line-height:1.7;margin-bottom:8px">Here's what happens next:</p>
    <ul style="color:#666;font-size:14px;line-height:2;margin:0 0 24px;padding-left:20px">
      <li>We'll email you <strong style="color:#1A1A1A">before anyone else</strong> when we launch</li>
      <li>Early members get <strong style="color:#1A1A1A">priority access</strong> and exclusive perks</li>
      <li>Tell a friend — they'll thank you later</li>
    </ul>
    <div style="text-align:center;margin-bottom:24px">
      <a href="https://qent.online" style="display:inline-block;padding:14px 36px;background:#1A1A1A;color:#fff;border-radius:100px;font-size:15px;font-weight:700;text-decoration:none">Visit Qent</a>
    </div>
    <p style="color:#999;font-size:13px;text-align:center">Know someone who needs a car? Or has one sitting idle? Forward this.</p>
  </div>
  <div style="text-align:center;padding:24px 32px;color:#999;font-size:12px;border-top:1px solid #eee">
    <p>Qent - Car Rental Made Easy</p>
    <p>&copy; 2026 Qent. Lagos, Nigeria.</p>
  </div>
</div>
</body>
</html>"#,
        greeting = if name.is_empty() {
            String::new()
        } else {
            format!(" {}", name)
        },
        early_label = early_label,
        role_line = role_line,
    );

    let client = reqwest::Client::new();
    let result = client
        .post("https://api.resend.com/emails")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "from": "Qent <noreply@qent.online>",
            "to": [email],
            "subject": "You're in — Welcome to Qent",
            "html": html,
        }))
        .send()
        .await;

    match result {
        Ok(resp) => {
            if resp.status().is_success() {
                log::info!("Waitlist email sent to {}", email);
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                log::error!("Resend API error {}: {}", status, body);
            }
        }
        Err(e) => log::error!("Failed to send waitlist email: {}", e),
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct WaitlistRequest {
    pub email: String,
    pub phone: Option<String>,
    pub name: Option<String>,
    pub role: Option<String>,
    pub city: Option<String>,
    pub referral_code: Option<String>,
}

/// POST /api/waitlist — Join the waitlist (public, no auth). Sends a welcome email asynchronously.
#[utoipa::path(
    post,
    path = "/api/waitlist",
    tag = "Waitlist",
    request_body = WaitlistRequest,
    responses(
        (status = 200, description = "{ message, position }"),
        (status = 400, description = "Invalid email"),
    ),
)]
pub async fn join_waitlist(
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    body: web::Json<WaitlistRequest>,
) -> HttpResponse {
    let email = body.email.trim().to_lowercase();
    if !email.contains('@') || !email.contains('.') {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Please enter a valid email"}));
    }

    let role = body.role.as_deref().unwrap_or("renter");

    let result = sqlx::query(
        r#"INSERT INTO waitlist (email, phone, name, role, city, referral_code)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (email) DO UPDATE SET
            phone = COALESCE(EXCLUDED.phone, waitlist.phone),
            name = COALESCE(EXCLUDED.name, waitlist.name),
            role = EXCLUDED.role,
            city = COALESCE(EXCLUDED.city, waitlist.city)"#,
    )
    .bind(&email)
    .bind(&body.phone)
    .bind(&body.name)
    .bind(role)
    .bind(&body.city)
    .bind(&body.referral_code)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => {
            // Get total count for social proof
            let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM waitlist")
                .fetch_one(pool.get_ref())
                .await
                .unwrap_or(0);

            // Send welcome email (fire and forget — don't block the response)
            let api_key = config.resend_api_key.clone();
            let email_to = email.clone();
            let name = body.name.clone().unwrap_or_default();
            let role_for_email = role.to_string();
            let pos = count;
            tokio::spawn(async move {
                send_waitlist_email(&api_key, &email_to, &name, pos, &role_for_email).await;
            });

            HttpResponse::Ok().json(serde_json::json!({
                "message": "You're on the list!",
                "position": count
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// GET /api/waitlist/count — Public count for social proof
#[utoipa::path(
    get,
    path = "/api/waitlist/count",
    tag = "Waitlist",
    responses(
        (status = 200, description = "{ count }"),
    ),
)]
pub async fn waitlist_count(pool: web::Data<PgPool>) -> HttpResponse {
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM waitlist")
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);

    HttpResponse::Ok().json(serde_json::json!({"count": count}))
}

/// GET /api/admin/waitlist — Full list of waitlist signups, newest first.
/// Admin-only. Used by the admin dashboard so we can see who's waiting.
#[utoipa::path(
    get,
    path = "/api/admin/waitlist",
    tag = "Admin",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Waitlist entries newest first"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
    ),
)]
pub async fn admin_list_waitlist(
    req: actix_web::HttpRequest,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    if let Err(resp) = crate::handlers::admin::require_admin(&req) {
        return resp;
    }
    let rows: Vec<(uuid::Uuid, String, Option<String>, Option<String>, String, Option<String>, Option<String>, chrono::NaiveDateTime)> =
        sqlx::query_as(
            r#"SELECT id, email, phone, name, role, city, referral_code, created_at
               FROM waitlist
               ORDER BY created_at DESC"#,
        )
        .fetch_all(pool.get_ref())
        .await
        .unwrap_or_default();
    let json: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(id, email, phone, name, role, city, referral_code, created_at)| {
            serde_json::json!({
                "id": id.to_string(),
                "email": email,
                "phone": phone,
                "name": name,
                "role": role,
                "city": city,
                "referral_code": referral_code,
                "created_at": created_at,
            })
        })
        .collect();
    HttpResponse::Ok().json(json)
}
