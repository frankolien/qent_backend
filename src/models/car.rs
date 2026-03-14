use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "car_status", rename_all = "lowercase")]
pub enum CarStatus {
    Active,
    Inactive,
    PendingApproval,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Car {
    pub id: Uuid,
    pub host_id: Uuid,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub color: String,
    pub plate_number: String,
    pub description: String,
    pub price_per_day: f64,
    pub location: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub photos: Vec<String>,
    pub features: Vec<String>,
    pub status: CarStatus,
    pub seats: i32,
    pub rating: Option<f64>,
    pub trip_count: Option<i64>,
    pub available_from: Option<NaiveDate>,
    pub available_to: Option<NaiveDate>,
    pub views_count: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub host_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCarRequest {
    #[validate(length(min = 1))]
    pub make: String,
    #[validate(length(min = 1))]
    pub model: String,
    pub year: i32,
    pub color: String,
    pub plate_number: String,
    pub description: String,
    pub price_per_day: f64,
    pub location: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub photos: Vec<String>,
    pub features: Option<Vec<String>>,
    pub seats: Option<i32>,
    pub available_from: Option<NaiveDate>,
    pub available_to: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCarRequest {
    pub description: Option<String>,
    pub price_per_day: Option<f64>,
    pub location: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub photos: Option<Vec<String>>,
    pub features: Option<Vec<String>>,
    pub available_from: Option<NaiveDate>,
    pub available_to: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct CarSearchQuery {
    pub location: Option<String>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub color: Option<String>,
    pub seats: Option<i32>,
    pub fuel_type: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
