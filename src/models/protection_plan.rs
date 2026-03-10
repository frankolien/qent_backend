use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "plan_tier", rename_all = "lowercase")]
pub enum PlanTier {
    Basic,
    Standard,
    Premium,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProtectionPlan {
    pub id: Uuid,
    pub name: String,
    pub tier: PlanTier,
    pub description: String,
    pub daily_rate: f64,
    pub coverage_amount: f64,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
}
