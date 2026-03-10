use actix_web::{web, HttpResponse};
use sqlx::PgPool;

use crate::models::ProtectionPlan;

pub async fn list_plans(pool: web::Data<PgPool>) -> HttpResponse {
    let result = sqlx::query_as::<_, ProtectionPlan>(
        "SELECT * FROM protection_plans WHERE is_active = true ORDER BY daily_rate ASC",
    )
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(plans) => HttpResponse::Ok().json(plans),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}
