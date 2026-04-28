use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, ToSchema)]
#[sqlx(type_name = "booking_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum BookingStatus {
    Pending,
    Approved,
    Rejected,
    Confirmed,
    Active,
    Completed,
    Cancelled,
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
