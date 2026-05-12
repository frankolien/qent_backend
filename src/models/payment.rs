use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct DamageReport {
    pub id: Uuid,
    pub booking_id: Uuid,
    pub reporter_id: Uuid,
    pub reporter_role: String,
    pub photos: Vec<String>,
    pub notes: Option<String>,
    pub odometer_reading: Option<i32>,
    pub fuel_level: Option<String>,
    pub exterior_condition: String,
    pub interior_condition: String,
    pub confirmed: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDamageReportRequest {
    pub booking_id: Uuid,
    pub photos: Vec<String>,
    pub notes: Option<String>,
    pub odometer_reading: Option<i32>,
    pub fuel_level: Option<String>,
    pub exterior_condition: Option<String>,
    pub interior_condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, ToSchema)]
#[sqlx(type_name = "payment_status", rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Success,
    Failed,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, ToSchema)]
#[sqlx(type_name = "transaction_type", rename_all = "lowercase")]
pub enum TransactionType {
    Payment,
    Payout,
    Refund,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Payment {
    pub id: Uuid,
    pub booking_id: Uuid,
    pub payer_id: Uuid,
    pub amount: f64,
    pub currency: String,
    pub provider: String,
    pub provider_reference: Option<String>,
    pub status: PaymentStatus,
    pub transaction_type: TransactionType,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct InitiatePaymentRequest {
    pub booking_id: Uuid,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaymentInitResponse {
    pub authorization_url: String,
    pub reference: String,
}

#[derive(Debug, Deserialize)]
pub struct PaystackWebhookEvent {
    pub event: String,
    pub data: PaystackWebhookData,
}

#[derive(Debug, Deserialize)]
pub struct PaystackWebhookData {
    pub reference: String,
    pub status: String,
    pub amount: i64,
    pub currency: String,
    pub authorization: Option<PaystackAuthorization>,
}

#[derive(Debug, Deserialize)]
pub struct PaystackAuthorization {
    pub authorization_code: String,
    pub card_type: Option<String>,
    pub last4: Option<String>,
    pub exp_month: Option<String>,
    pub exp_year: Option<String>,
    pub bin: Option<String>,
    pub bank: Option<String>,
    pub brand: Option<String>,
    pub reusable: Option<bool>,
    pub account_name: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PayoutRequest {
    pub amount: f64,
    pub bank_code: String,
    pub account_number: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct VerifyAccountRequest {
    pub account_number: String,
    pub bank_code: String,
}

#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct EarningsStats {
    pub total_earned: f64,
    pub this_month: f64,
    pub pending_earnings: f64,
    pub completed_trips: i32,
}

#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct EarningEntry {
    pub booking_id: Uuid,
    pub car_name: Option<String>,
    pub earned: f64,
    pub completed_at: NaiveDateTime,
    pub renter_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct WalletTransaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: f64,
    pub balance_after: f64,
    pub description: String,
    pub reference_id: Option<Uuid>,
    pub status: String,
    pub admin_notes: Option<String>,
    pub created_at: NaiveDateTime,
}
