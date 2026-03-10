use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Review {
    pub id: Uuid,
    pub booking_id: Uuid,
    pub reviewer_id: Uuid,
    pub reviewee_id: Uuid,
    pub rating: i32,
    pub comment: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateReviewRequest {
    pub booking_id: Uuid,
    pub reviewee_id: Uuid,
    #[validate(range(min = 1, max = 5))]
    pub rating: i32,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserRatingSummary {
    pub user_id: Uuid,
    pub average_rating: f64,
    pub total_reviews: i64,
}
