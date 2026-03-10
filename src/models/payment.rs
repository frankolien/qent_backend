use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "payment_status", rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Success,
    Failed,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "transaction_type", rename_all = "lowercase")]
pub enum TransactionType {
    Payment,
    Payout,
    Refund,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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

#[derive(Debug, Deserialize)]
pub struct InitiatePaymentRequest {
    pub booking_id: Uuid,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Deserialize)]
pub struct PayoutRequest {
    pub amount: f64,
    pub bank_code: String,
    pub account_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WalletTransaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: f64,
    pub balance_after: f64,
    pub description: String,
    pub reference_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
}
