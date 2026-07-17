use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    Booking, BookingAction, BookingActionRequest, BookingStatus, Claims, UserRole,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionSource {
    HostAction,
    AutoComplete,
}

#[derive(Debug, thiserror::Error)]
pub enum BookingWorkflowError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("booking not found")]
    BookingNotFound,
    #[error("booking not active")]
    BookingNotActive,
    #[error("forbidden: {0}")]
    Forbidden(&'static str),
    #[error("invalid transition: {0}")]
    InvalidTransition(&'static str),
}

#[derive(Debug)]
pub struct BookingTransitionResult {
    pub before: Booking,
    pub after: Booking,
}

#[derive(Debug)]
pub struct PaymentConfirmationResult {
    pub payment_id: Uuid,
    pub booking_id: Uuid,
    pub payer_id: Uuid,
    pub changed: bool,
}

/// Central booking workflow mutations that must stay consistent across
/// handlers and background jobs.
pub struct BookingWorkflows;

impl BookingWorkflows {
    pub async fn confirm_payment_received(
        pool: &PgPool,
        payment_id: Uuid,
        booking_id: Uuid,
        payer_id: Uuid,
    ) -> Result<PaymentConfirmationResult, BookingWorkflowError> {
        let mut tx = pool.begin().await?;
        let flipped = sqlx::query(
            "UPDATE payments SET status = 'success', confirmed_at = NOW() WHERE id = $1 AND status = 'broadcast'",
        )
        .bind(payment_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();

        if flipped == 0 {
            tx.rollback().await?;
            return Ok(PaymentConfirmationResult {
                payment_id,
                booking_id,
                payer_id,
                changed: false,
            });
        }

        sqlx::query(
            "UPDATE bookings SET status = 'paid', updated_at = NOW() WHERE id = $1 AND status = 'pending_payment'",
        )
        .bind(booking_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(PaymentConfirmationResult {
            payment_id,
            booking_id,
            payer_id,
            changed: true,
        })
    }

    pub async fn apply_action(
        pool: &PgPool,
        booking_id: Uuid,
        actor: &Claims,
        request: &BookingActionRequest,
    ) -> Result<BookingTransitionResult, BookingWorkflowError> {
        let booking = sqlx::query_as::<_, Booking>("SELECT * FROM bookings WHERE id = $1")
            .bind(booking_id)
            .fetch_optional(pool)
            .await?
            .ok_or(BookingWorkflowError::BookingNotFound)?;

        let new_status = match request.action {
            BookingAction::Approve => {
                if booking.host_id != actor.sub && actor.role != UserRole::Admin {
                    return Err(BookingWorkflowError::Forbidden("Only the host can approve"));
                }
                if booking.status != BookingStatus::Pending {
                    return Err(BookingWorkflowError::InvalidTransition(
                        "Booking is not pending",
                    ));
                }
                BookingStatus::Approved
            }
            BookingAction::Reject => {
                if booking.host_id != actor.sub && actor.role != UserRole::Admin {
                    return Err(BookingWorkflowError::Forbidden("Only the host can reject"));
                }
                BookingStatus::Rejected
            }
            BookingAction::Cancel => {
                if booking.renter_id != actor.sub
                    && booking.host_id != actor.sub
                    && actor.role != UserRole::Admin
                {
                    return Err(BookingWorkflowError::Forbidden("Not authorized to cancel"));
                }
                BookingStatus::Cancelled
            }
            BookingAction::Activate => {
                if booking.host_id != actor.sub && actor.role != UserRole::Admin {
                    return Err(BookingWorkflowError::Forbidden(
                        "Only the host can activate",
                    ));
                }
                if booking.status != BookingStatus::Approved
                    && booking.status != BookingStatus::Confirmed
                    && booking.status != BookingStatus::Paid
                {
                    return Err(BookingWorkflowError::InvalidTransition(
                        "Booking must be paid to activate",
                    ));
                }
                BookingStatus::Active
            }
            BookingAction::Complete => {
                if booking.host_id != actor.sub && actor.role != UserRole::Admin {
                    return Err(BookingWorkflowError::Forbidden(
                        "Only the host can complete",
                    ));
                }
                if booking.status != BookingStatus::Active {
                    return Err(BookingWorkflowError::InvalidTransition(
                        "Booking is not active",
                    ));
                }

                Self::complete_booking(pool, booking_id, CompletionSource::HostAction).await?;
                let after = sqlx::query_as::<_, Booking>("SELECT * FROM bookings WHERE id = $1")
                    .bind(booking_id)
                    .fetch_one(pool)
                    .await?;
                return Ok(BookingTransitionResult {
                    before: booking,
                    after,
                });
            }
        };

        let after = sqlx::query_as::<_, Booking>(
            r#"UPDATE bookings SET status = $1, cancellation_reason = $2, updated_at = NOW()
            WHERE id = $3 RETURNING *"#,
        )
        .bind(&new_status)
        .bind(&request.reason)
        .bind(booking_id)
        .fetch_one(pool)
        .await?;

        Ok(BookingTransitionResult {
            before: booking,
            after,
        })
    }

    /// Complete an active booking and atomically credit the host payout.
    ///
    /// Today this preserves the existing legacy payout math (85% of
    /// subtotal) so the refactor improves structure without changing
    /// behaviour. If/when the business rule changes, change it here once.
    pub async fn complete_booking(
        pool: &PgPool,
        booking_id: Uuid,
        source: CompletionSource,
    ) -> Result<(), BookingWorkflowError> {
        let mut tx = pool.begin().await?;

        let booking = sqlx::query_as::<_, (Uuid, Uuid, f64, String)>(
            "SELECT id, host_id, subtotal, status FROM bookings WHERE id = $1",
        )
        .bind(booking_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(BookingWorkflowError::BookingNotFound)?;

        let (_id, host_id, subtotal, status) = booking;
        if status != "active" {
            return Err(BookingWorkflowError::BookingNotActive);
        }

        sqlx::query(
            "UPDATE bookings SET status = 'completed', updated_at = NOW() WHERE id = $1 AND status = 'active'",
        )
        .bind(booking_id)
        .execute(&mut *tx)
        .await?;

        let host_payout = subtotal * 0.85;

        sqlx::query(
            "UPDATE users SET wallet_balance = wallet_balance + $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(host_payout)
        .bind(host_id)
        .execute(&mut *tx)
        .await?;

        let description = match source {
            CompletionSource::HostAction => format!("Payout for booking {}", booking_id),
            CompletionSource::AutoComplete => format!("Auto-payout for booking {}", booking_id),
        };

        sqlx::query(
            r#"INSERT INTO wallet_transactions (id, user_id, amount, balance_after, description, reference_id, created_at)
            VALUES ($1, $2, $3, (SELECT wallet_balance FROM users WHERE id = $2), $4, $5, NOW())"#,
        )
        .bind(Uuid::new_v4())
        .bind(host_id)
        .bind(host_payout)
        .bind(description)
        .bind(booking_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use sqlx::postgres::PgPoolOptions;

    async fn test_pool() -> Option<PgPool> {
        let database_url = std::env::var("DATABASE_URL").ok()?;
        PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .ok()
    }

    fn claims(user_id: Uuid, role: UserRole) -> Claims {
        Claims {
            sub: user_id,
            role,
            exp: 4_102_444_800,
        }
    }

    async fn seed_user(pool: &PgPool, id: Uuid, role: UserRole, email: &str) {
        sqlx::query(
            r#"INSERT INTO users (
                id, email, password_hash, full_name, role, verification_status,
                wallet_balance, is_active, created_at, updated_at
            ) VALUES ($1, $2, 'pw', $3, $4, 'pending', 0.0, true, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(email)
        .bind(format!("user-{}", id))
        .bind(role)
        .execute(pool)
        .await
        .unwrap();
    }

    async fn seed_booking(
        pool: &PgPool,
        booking_id: Uuid,
        renter_id: Uuid,
        host_id: Uuid,
        status: &str,
    ) {
        let car_id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO cars (
                id, host_id, make, model, year, color, plate_number, description,
                price_per_day, location, photos, features, status, created_at, updated_at
            ) VALUES (
                $1, $2, 'Toyota', 'Camry', 2020, 'Black', $3, 'Nice car',
                100.0, 'Lagos', '{}', '{}', 'active', NOW(), NOW()
            )"#,
        )
        .bind(car_id)
        .bind(host_id)
        .bind(format!("PLATE-{}", booking_id))
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            r#"INSERT INTO bookings (
                id, car_id, renter_id, host_id, start_date, end_date, total_days,
                price_per_day, subtotal, protection_fee, service_fee, total_amount,
                status, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, 2,
                100.0, 200.0, 0.0, 20.0, 220.0,
                $7, NOW(), NOW()
            )"#,
        )
        .bind(booking_id)
        .bind(car_id)
        .bind(renter_id)
        .bind(host_id)
        .bind(NaiveDate::from_ymd_opt(2026, 1, 10).unwrap())
        .bind(NaiveDate::from_ymd_opt(2026, 1, 12).unwrap())
        .bind(status)
        .execute(pool)
        .await
        .unwrap();
    }

    async fn seed_payment(
        pool: &PgPool,
        payment_id: Uuid,
        booking_id: Uuid,
        payer_id: Uuid,
        status: &str,
    ) {
        sqlx::query(
            r#"INSERT INTO payments (
                id, booking_id, payer_id, amount, currency, provider,
                provider_reference, status, transaction_type, created_at,
                chain, amount_usdc, to_address
            ) VALUES (
                $1, $2, $3, 224.0, 'USDC', 'privy',
                NULL, $4, 'payment', NOW(),
                'base', 224.0, '0xescrow'
            )"#,
        )
        .bind(payment_id)
        .bind(booking_id)
        .bind(payer_id)
        .bind(status)
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn approve_requires_host_or_admin() {
        let Some(pool) = test_pool().await else {
            return;
        };

        let renter_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let stranger_id = Uuid::new_v4();
        let booking_id = Uuid::new_v4();

        seed_user(&pool, renter_id, UserRole::Renter, "wf_renter1@test.local").await;
        seed_user(&pool, host_id, UserRole::Host, "wf_host1@test.local").await;
        seed_user(
            &pool,
            stranger_id,
            UserRole::Renter,
            "wf_stranger1@test.local",
        )
        .await;
        seed_booking(&pool, booking_id, renter_id, host_id, "pending").await;

        let err = BookingWorkflows::apply_action(
            &pool,
            booking_id,
            &claims(stranger_id, UserRole::Renter),
            &BookingActionRequest {
                action: BookingAction::Approve,
                reason: None,
            },
        )
        .await
        .unwrap_err();

        assert!(matches!(err, BookingWorkflowError::Forbidden(_)));
    }

    #[tokio::test]
    async fn approve_moves_pending_to_approved() {
        let Some(pool) = test_pool().await else {
            return;
        };

        let renter_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let booking_id = Uuid::new_v4();

        seed_user(&pool, renter_id, UserRole::Renter, "wf_renter2@test.local").await;
        seed_user(&pool, host_id, UserRole::Host, "wf_host2@test.local").await;
        seed_booking(&pool, booking_id, renter_id, host_id, "pending").await;

        let result = BookingWorkflows::apply_action(
            &pool,
            booking_id,
            &claims(host_id, UserRole::Host),
            &BookingActionRequest {
                action: BookingAction::Approve,
                reason: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(result.before.status, BookingStatus::Pending);
        assert_eq!(result.after.status, BookingStatus::Approved);
    }

    #[tokio::test]
    async fn activate_requires_paid_like_status() {
        let Some(pool) = test_pool().await else {
            return;
        };

        let renter_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let booking_id = Uuid::new_v4();

        seed_user(&pool, renter_id, UserRole::Renter, "wf_renter3@test.local").await;
        seed_user(&pool, host_id, UserRole::Host, "wf_host3@test.local").await;
        seed_booking(&pool, booking_id, renter_id, host_id, "pending").await;

        let err = BookingWorkflows::apply_action(
            &pool,
            booking_id,
            &claims(host_id, UserRole::Host),
            &BookingActionRequest {
                action: BookingAction::Activate,
                reason: None,
            },
        )
        .await
        .unwrap_err();

        assert!(matches!(err, BookingWorkflowError::InvalidTransition(_)));
    }

    #[tokio::test]
    async fn complete_booking_credits_host_and_records_wallet_tx() {
        let Some(pool) = test_pool().await else {
            return;
        };

        let renter_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let booking_id = Uuid::new_v4();

        seed_user(&pool, renter_id, UserRole::Renter, "wf_renter4@test.local").await;
        seed_user(&pool, host_id, UserRole::Host, "wf_host4@test.local").await;
        seed_booking(&pool, booking_id, renter_id, host_id, "active").await;

        BookingWorkflows::complete_booking(&pool, booking_id, CompletionSource::HostAction)
            .await
            .unwrap();

        let status = sqlx::query_scalar::<_, String>("SELECT status FROM bookings WHERE id = $1")
            .bind(booking_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        let balance =
            sqlx::query_scalar::<_, f64>("SELECT wallet_balance FROM users WHERE id = $1")
                .bind(host_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        let wallet_tx_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM wallet_transactions WHERE user_id = $1 AND reference_id = $2",
        )
        .bind(host_id)
        .bind(booking_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(status, "completed");
        assert!((balance - 170.0).abs() < f64::EPSILON);
        assert_eq!(wallet_tx_count, 1);
    }

    #[tokio::test]
    async fn confirm_payment_received_marks_payment_and_booking_once() {
        let Some(pool) = test_pool().await else {
            return;
        };

        let renter_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let booking_id = Uuid::new_v4();
        let payment_id = Uuid::new_v4();

        seed_user(&pool, renter_id, UserRole::Renter, "wf_renter5@test.local").await;
        seed_user(&pool, host_id, UserRole::Host, "wf_host5@test.local").await;
        seed_booking(&pool, booking_id, renter_id, host_id, "pending_payment").await;
        seed_payment(&pool, payment_id, booking_id, renter_id, "broadcast").await;

        let result =
            BookingWorkflows::confirm_payment_received(&pool, payment_id, booking_id, renter_id)
                .await
                .unwrap();
        let second =
            BookingWorkflows::confirm_payment_received(&pool, payment_id, booking_id, renter_id)
                .await
                .unwrap();

        let payment_status =
            sqlx::query_scalar::<_, String>("SELECT status FROM payments WHERE id = $1")
                .bind(payment_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        let booking_status =
            sqlx::query_scalar::<_, String>("SELECT status FROM bookings WHERE id = $1")
                .bind(booking_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert!(result.changed);
        assert!(!second.changed);
        assert_eq!(payment_status, "success");
        assert_eq!(booking_status, "paid");
    }
}
