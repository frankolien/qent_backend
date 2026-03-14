use chrono::NaiveDate;

pub struct EmailService {
    api_key: String,
    from_email: String,
}

impl EmailService {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            from_email: "Qent <noreply@qent.online>".to_string(),
        }
    }

    pub async fn send_booking_confirmation(
        &self,
        to_email: &str,
        customer_name: &str,
        car_name: &str,
        booking_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        total_days: i32,
        subtotal: f64,
        service_fee: f64,
        protection_fee: f64,
        total_amount: f64,
        payment_reference: &str,
    ) {
        if self.api_key.is_empty() {
            log::warn!("Resend API key not set, skipping email");
            return;
        }

        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 0; background: #f5f5f5; }}
  .container {{ max-width: 600px; margin: 0 auto; background: #fff; }}
  .header {{ background: #1A1A1A; color: #fff; padding: 32px; text-align: center; }}
  .header h1 {{ margin: 0; font-size: 24px; font-weight: 700; }}
  .body {{ padding: 32px; }}
  .greeting {{ font-size: 18px; font-weight: 600; margin-bottom: 8px; }}
  .subtitle {{ color: #666; margin-bottom: 24px; }}
  .details {{ background: #f9f9f9; border-radius: 12px; padding: 24px; margin-bottom: 24px; }}
  .details h3 {{ margin: 0 0 16px 0; font-size: 16px; }}
  .row {{ display: flex; justify-content: space-between; padding: 8px 0; border-bottom: 1px solid #eee; }}
  .row:last-child {{ border-bottom: none; }}
  .label {{ color: #666; }}
  .value {{ font-weight: 600; }}
  .total-row {{ border-top: 2px dashed #ddd; padding-top: 12px; margin-top: 4px; }}
  .total-label {{ font-size: 16px; font-weight: 700; }}
  .total-value {{ font-size: 16px; font-weight: 700; }}
  .footer {{ text-align: center; padding: 24px 32px; color: #999; font-size: 12px; border-top: 1px solid #eee; }}
  .badge {{ display: inline-block; background: #E8F5E9; color: #2E7D32; padding: 4px 12px; border-radius: 20px; font-size: 13px; font-weight: 600; }}
</style>
</head>
<body>
<div class="container">
  <div class="header">
    <h1>Qent</h1>
  </div>
  <div class="body">
    <p class="greeting">Hi {customer_name},</p>
    <p class="subtitle">Your booking has been confirmed! Here's your receipt.</p>
    <p><span class="badge">Payment Confirmed</span></p>

    <div class="details">
      <h3>Booking Details</h3>
      <table width="100%" cellpadding="0" cellspacing="0">
        <tr><td style="color:#666;padding:8px 0">Booking ID</td><td style="font-weight:600;text-align:right;padding:8px 0">#{booking_short}</td></tr>
        <tr><td style="color:#666;padding:8px 0;border-top:1px solid #eee">Car</td><td style="font-weight:600;text-align:right;padding:8px 0;border-top:1px solid #eee">{car_name}</td></tr>
        <tr><td style="color:#666;padding:8px 0;border-top:1px solid #eee">Pick-up</td><td style="font-weight:600;text-align:right;padding:8px 0;border-top:1px solid #eee">{start_date}</td></tr>
        <tr><td style="color:#666;padding:8px 0;border-top:1px solid #eee">Return</td><td style="font-weight:600;text-align:right;padding:8px 0;border-top:1px solid #eee">{end_date}</td></tr>
        <tr><td style="color:#666;padding:8px 0;border-top:1px solid #eee">Duration</td><td style="font-weight:600;text-align:right;padding:8px 0;border-top:1px solid #eee">{total_days} day{day_s}</td></tr>
      </table>
    </div>

    <div class="details">
      <h3>Payment Summary</h3>
      <table width="100%" cellpadding="0" cellspacing="0">
        <tr><td style="color:#666;padding:8px 0">Subtotal</td><td style="font-weight:600;text-align:right;padding:8px 0">{naira}{subtotal}</td></tr>
        <tr><td style="color:#666;padding:8px 0;border-top:1px solid #eee">Service fee</td><td style="font-weight:600;text-align:right;padding:8px 0;border-top:1px solid #eee">{naira}{service_fee}</td></tr>
        {protection_row}
        <tr style="border-top:2px dashed #ddd"><td style="font-size:16px;font-weight:700;padding:12px 0 8px">Total</td><td style="font-size:16px;font-weight:700;text-align:right;padding:12px 0 8px">{naira}{total_amount}</td></tr>
      </table>
      <p style="color:#999;font-size:12px;margin:8px 0 0">Reference: {payment_reference}</p>
    </div>

    <p style="color:#666;font-size:14px">If you have any questions about your booking, feel free to reach out through the app.</p>
  </div>
  <div class="footer">
    <p>Qent - Car Rental Made Easy</p>
    <p>This is an automated email. Please do not reply.</p>
  </div>
</div>
</body>
</html>"#,
            customer_name = customer_name,
            car_name = car_name,
            booking_short = if booking_id.len() > 8 { &booking_id[..8] } else { booking_id },
            start_date = start_date.format("%d %b %Y"),
            end_date = end_date.format("%d %b %Y"),
            total_days = total_days,
            day_s = if total_days == 1 { "" } else { "s" },
            subtotal = format_naira(subtotal),
            service_fee = format_naira(service_fee),
            protection_row = if protection_fee > 0.0 {
                format!(r#"<tr><td style="color:#666;padding:8px 0;border-top:1px solid #eee">Protection fee</td><td style="font-weight:600;text-align:right;padding:8px 0;border-top:1px solid #eee">{naira}{fee}</td></tr>"#,
                    naira = "\u{20A6}",
                    fee = format_naira(protection_fee))
            } else {
                String::new()
            },
            total_amount = format_naira(total_amount),
            payment_reference = payment_reference,
            naira = "\u{20A6}",
        );

        let client = reqwest::Client::new();
        let result = client
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "from": self.from_email,
                "to": [to_email],
                "subject": format!("Booking Confirmed - {}", car_name),
                "html": html,
            }))
            .send()
            .await;

        match result {
            Ok(resp) => {
                if resp.status().is_success() {
                    log::info!("Booking confirmation email sent to {}", to_email);
                } else {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    log::error!("Resend API error {}: {}", status, body);
                }
            }
            Err(e) => log::error!("Failed to send email: {}", e),
        }
    }
}

fn format_naira(amount: f64) -> String {
    let int_amount = amount as i64;
    let formatted = int_amount
        .to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect::<Vec<&str>>()
        .join(",");
    formatted
}
