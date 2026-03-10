use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SavedCard {
    pub id: Uuid,
    pub user_id: Uuid,
    pub authorization_code: String,
    pub card_type: String,
    pub last4: String,
    pub exp_month: String,
    pub exp_year: String,
    pub bin: String,
    pub bank: Option<String>,
    pub brand: String,
    pub is_default: bool,
    pub cardholder_name: Option<String>,
    pub created_at: NaiveDateTime,
}

/// What we return to the client (no authorization_code exposed)
#[derive(Debug, Serialize)]
pub struct SavedCardPublic {
    pub id: Uuid,
    pub card_type: String,
    pub last4: String,
    pub exp_month: String,
    pub exp_year: String,
    pub brand: String,
    pub bank: Option<String>,
    pub is_default: bool,
    pub cardholder_name: Option<String>,
    pub created_at: NaiveDateTime,
}

impl From<SavedCard> for SavedCardPublic {
    fn from(c: SavedCard) -> Self {
        Self {
            id: c.id,
            card_type: c.card_type,
            last4: c.last4,
            exp_month: c.exp_month,
            exp_year: c.exp_year,
            brand: c.brand,
            bank: c.bank,
            is_default: c.is_default,
            cardholder_name: c.cardholder_name,
            created_at: c.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SetDefaultCardRequest {
    pub card_id: Uuid,
}
