use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

// V2 §5.2 — migration 027 converted bookings.status from enum to
// TEXT to add new V2 states (`pending_payment`, `paid`, `no_show`,
// `failed_handover`) without an ALTER TYPE rebuild. Decode/encode
// goes through Postgres TEXT now; the legacy enum variants stay
// for back-compat reads of old rows.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, ToSchema)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum BookingStatus {
    // V2 states (§5.2)
    PendingPayment,
    Paid,
    Active,
    Completed,
    Cancelled,
    Refunded,
    Disputed,
    NoShow,
    FailedHandover,
    // Legacy V1 states tolerated during transition
    Pending,
    Approved,
    Rejected,
    Confirmed,
    InProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Booking {
    pub id: Uuid,
    pub car_id: Uuid,
    pub renter_id: Uuid,
    pub host_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub total_days: i32,
    pub price_per_day: f64,
    pub subtotal: f64,
    pub protection_plan_id: Option<Uuid>,
    pub protection_fee: f64,
    pub service_fee: f64,
    pub total_amount: f64,
    // V2 §9.2 — USDC-denominated charges (null on legacy V1 rows).
    // Serialized as JSON strings to preserve full NUMERIC(20,6) precision;
    // `value_type = String` tells utoipa to document them as strings.
    #[schema(value_type = String)]
    pub host_price_usdc: Option<rust_decimal::Decimal>,
    #[schema(value_type = String)]
    pub service_fee_usdc: Option<rust_decimal::Decimal>,
    #[schema(value_type = String)]
    pub protection_price_usdc: Option<rust_decimal::Decimal>,
    #[schema(value_type = String)]
    pub total_usdc: Option<rust_decimal::Decimal>,
    pub status: BookingStatus,
    pub cancellation_reason: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct BookingWithCar {
    pub id: Uuid,
    pub car_id: Uuid,
    pub renter_id: Uuid,
    pub host_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub total_days: i32,
    pub price_per_day: f64,
    pub subtotal: f64,
    pub protection_plan_id: Option<Uuid>,
    pub protection_fee: f64,
    pub service_fee: f64,
    pub total_amount: f64,
    // V2 §9.2 — USDC-denominated charges (null on legacy V1 rows).
    // Serialized as JSON strings to preserve full NUMERIC(20,6) precision;
    // `value_type = String` tells utoipa to document them as strings.
    #[schema(value_type = String)]
    pub host_price_usdc: Option<rust_decimal::Decimal>,
    #[schema(value_type = String)]
    pub service_fee_usdc: Option<rust_decimal::Decimal>,
    #[schema(value_type = String)]
    pub protection_price_usdc: Option<rust_decimal::Decimal>,
    #[schema(value_type = String)]
    pub total_usdc: Option<rust_decimal::Decimal>,
    pub status: BookingStatus,
    pub cancellation_reason: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub car_name: Option<String>,
    pub car_photo: Option<String>,
    pub car_location: Option<String>,
    pub renter_name: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateBookingRequest {
    pub car_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub protection_plan_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BookingActionRequest {
    pub action: BookingAction,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum BookingAction {
    Approve,
    Reject,
    Cancel,
    Activate,
    Complete,
}
