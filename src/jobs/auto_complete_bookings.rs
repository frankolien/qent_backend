use sqlx::PgPool;
use uuid::Uuid;

use crate::services::booking_workflows::{
    BookingWorkflowError, BookingWorkflows, CompletionSource,
};

/// Background task: auto-complete bookings past their end_date (runs every hour).
pub async fn run(pool: PgPool) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
    loop {
        interval.tick().await;
        log::info!("Running auto-complete check for overdue bookings...");

        let overdue = sqlx::query_scalar::<_, Uuid>(
            r#"SELECT id FROM bookings
               WHERE status = 'active' AND end_date < CURRENT_DATE"#,
        )
        .fetch_all(&pool)
        .await;

        match overdue {
            Ok(bookings) => {
                let booking_count = bookings.len();
                for booking_id in bookings {
                    match BookingWorkflows::complete_booking(
                        &pool,
                        booking_id,
                        CompletionSource::AutoComplete,
                    )
                    .await
                    {
                        Ok(()) => {
                            let _ = sqlx::query(
                                r#"INSERT INTO notifications (id, user_id, title, message, notification_type, is_read, data, created_at)
                                VALUES ($1, (SELECT renter_id FROM bookings WHERE id = $2), 'Trip Completed',
                                'Your trip has been auto-completed. Leave a review!', 'booking_completed', false, $3, NOW())"#,
                            )
                            .bind(Uuid::new_v4())
                            .bind(booking_id)
                            .bind(serde_json::json!({"booking_id": booking_id.to_string()}))
                            .execute(&pool)
                            .await;
                        }
                        Err(BookingWorkflowError::BookingNotActive) => {
                            log::info!(
                                "Skipping auto-complete for booking {} because it is no longer active",
                                booking_id
                            );
                        }
                        Err(e) => {
                            log::error!("Auto-complete failed for booking {}: {}", booking_id, e);
                        }
                    }
                }
                if booking_count > 0 {
                    log::info!("Auto-completed {} overdue booking(s)", booking_count);
                }
            }
            Err(e) => log::error!("Auto-complete query failed: {}", e),
        }
    }
}
