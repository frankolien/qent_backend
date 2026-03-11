use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PartnerApplicationStatus {
    Pending,
    Approved,
    Rejected,
}

impl std::fmt::Display for PartnerApplicationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartnerApplicationStatus::Pending => write!(f, "pending"),
            PartnerApplicationStatus::Approved => write!(f, "approved"),
            PartnerApplicationStatus::Rejected => write!(f, "rejected"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PartnerApplication {
    pub id: Uuid,
    pub user_id: Uuid,
    pub full_name: String,
    pub email: String,
    pub phone: String,
    pub drivers_license: String,
    pub car_make: String,
    pub car_model: String,
    pub car_year: i32,
    pub car_color: String,
    pub car_plate_number: String,
    pub car_photos: Vec<String>,
    pub car_description: String,
    pub fuel_type: String,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePartnerApplicationRequest {
    #[validate(length(min = 2))]
    pub full_name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 5))]
    pub phone: String,
    #[validate(length(min = 1))]
    pub drivers_license: String,
    #[validate(length(min = 1))]
    pub car_make: String,
    #[validate(length(min = 1))]
    pub car_model: String,
    pub car_year: i32,
    #[validate(length(min = 1))]
    pub car_color: String,
    #[validate(length(min = 1))]
    pub car_plate_number: String,
    pub car_photos: Vec<String>,
    pub car_description: Option<String>,
    pub fuel_type: Option<String>,
    pub price_per_day: f64,
    pub location: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct HostDashboard {
    pub total_earnings: f64,
    pub active_listings: i64,
    pub completed_bookings: i64,
    pub average_rating: f64,
}
