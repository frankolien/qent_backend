use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Booking, BookingStatus, Claims};
use crate::services::email::EmailService;
use crate::services::push::PushService;

pub struct BookingNotifications<'a> {
    pub pool: &'a PgPool,
    pub push: Option<&'a PushService>,
    pub email: &'a EmailService,
}

impl<'a> BookingNotifications<'a> {
    pub async fn send_status_changed(
        &self,
        actor: &Claims,
        before: &Booking,
        after: &Booking,
        car_name: &str,
    ) {
        let Some(effect) = build_effect(actor, before, after, car_name) else {
            return;
        };

        if let Err(e) = self
            .create_notification(
                effect.notify_user_id,
                effect.title,
                &effect.message,
                effect.notification_type,
                Some(serde_json::json!({"booking_id": after.id.to_string()})),
            )
            .await
        {
            log::warn!(
                "failed to create booking notification for booking {}: {}",
                after.id,
                e
            );
        }

        let user_info = sqlx::query_as::<_, (String, String)>(
            "SELECT email, full_name FROM users WHERE id = $1",
        )
        .bind(effect.notify_user_id)
        .fetch_optional(self.pool)
        .await;

        if let Ok(Some((email, name))) = user_info {
            self.email
                .send_status_email(
                    &email,
                    &name,
                    car_name,
                    effect.status_key,
                    &effect.email_message,
                )
                .await;
        }
    }

    pub async fn create_notification(
        &self,
        user_id: Uuid,
        title: &'static str,
        message: &str,
        notification_type: &'static str,
        data: Option<serde_json::Value>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO notifications (id, user_id, title, message, notification_type, is_read, data, created_at)
            VALUES ($1, $2, $3, $4, $5, false, $6, NOW())"#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(title)
        .bind(message)
        .bind(notification_type)
        .bind(data.clone())
        .execute(self.pool)
        .await?;

        if let Some(push) = self.push {
            let payload = data.unwrap_or_else(|| serde_json::json!({}));
            let pool = self.pool.clone();
            let push = push.clone();
            let title = title.to_string();
            let message = message.to_string();
            tokio::spawn(async move {
                push.send_to_user(&pool, user_id, &title, &message, payload)
                    .await;
            });
        }

        Ok(())
    }
}

struct BookingStatusEffect {
    notify_user_id: Uuid,
    title: &'static str,
    message: String,
    email_message: String,
    notification_type: &'static str,
    status_key: &'static str,
}

fn build_effect(
    actor: &Claims,
    before: &Booking,
    after: &Booking,
    car_name: &str,
) -> Option<BookingStatusEffect> {
    match after.status {
        BookingStatus::Approved => Some(BookingStatusEffect {
            notify_user_id: before.renter_id,
            title: "Booking Approved",
            message: format!(
                "Your booking for {} has been approved! Coordinate pickup with the host.",
                car_name
            ),
            email_message: format!(
                "Your booking for {} has been approved! Coordinate pickup with the host.",
                car_name
            ),
            notification_type: "booking_approved",
            status_key: "approved",
        }),
        BookingStatus::Rejected => Some(BookingStatusEffect {
            notify_user_id: before.renter_id,
            title: "Booking Declined",
            message: format!("Your booking for {} was declined by the host.", car_name),
            email_message: format!("Your booking for {} was declined by the host.", car_name),
            notification_type: "booking_rejected",
            status_key: "rejected",
        }),
        BookingStatus::Cancelled => Some(BookingStatusEffect {
            notify_user_id: if actor.sub == before.renter_id {
                before.host_id
            } else {
                before.renter_id
            },
            title: "Booking Cancelled",
            message: format!("A booking for {} has been cancelled.", car_name),
            email_message: format!("A booking for {} has been cancelled.", car_name),
            notification_type: "booking_cancelled",
            status_key: "cancelled",
        }),
        BookingStatus::Active => Some(BookingStatusEffect {
            notify_user_id: before.renter_id,
            title: "Trip Started",
            message: format!(
                "Your trip with {} is now active. Enjoy your ride!",
                car_name
            ),
            email_message: format!(
                "Your trip with {} is now active. Enjoy your ride!",
                car_name
            ),
            notification_type: "booking_active",
            status_key: "active",
        }),
        BookingStatus::Completed => Some(BookingStatusEffect {
            notify_user_id: before.renter_id,
            title: "Trip Completed",
            message: format!("Your trip with {} is complete. Leave a review!", car_name),
            email_message: format!(
                "Your trip with {} is complete. We'd love your feedback!",
                car_name
            ),
            notification_type: "booking_completed",
            status_key: "completed",
        }),
        _ => None,
    }
}
