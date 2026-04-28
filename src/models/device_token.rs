use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct DeviceToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub platform: String,
    pub created_at: NaiveDateTime,
    pub last_seen_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterDeviceTokenRequest {
    pub token: String,
    pub platform: String,
}
