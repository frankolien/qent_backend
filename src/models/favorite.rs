use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Favorite {
    pub id: Uuid,
    pub user_id: Uuid,
    pub car_id: Uuid,
    pub created_at: NaiveDateTime,
}
