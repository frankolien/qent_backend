//! One-shot tool: inspect _sqlx_migrations and (with --reset-018) delete row 18.
//! Run with:  cargo run --bin migration_check
//!     or:    cargo run --bin migration_check -- --reset-018

use sqlx::postgres::PgPoolOptions;
use sqlx::Row;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await?;

    let rows = sqlx::query(
        "SELECT version, description, installed_on, success, checksum
         FROM _sqlx_migrations
         ORDER BY version",
    )
    .fetch_all(&pool)
    .await?;

    println!("\n=== _sqlx_migrations rows ===");
    println!("{:<8} {:<40} {:<22} {}", "version", "description", "installed_on", "success");
    for row in &rows {
        let v: i64 = row.get("version");
        let d: String = row.get("description");
        let installed_on: chrono::DateTime<chrono::Utc> = row.get("installed_on");
        let success: bool = row.get("success");
        println!("{:<8} {:<40} {:<22} {}", v, d, installed_on.format("%Y-%m-%d %H:%M:%S"), success);
    }

    let reset = std::env::args().any(|a| a == "--reset-018");
    if reset {
        let result = sqlx::query("DELETE FROM _sqlx_migrations WHERE version = 18")
            .execute(&pool)
            .await?;
        println!("\nDeleted {} row(s) for version 18.", result.rows_affected());

        let drop = sqlx::query("DROP TABLE IF EXISTS device_tokens")
            .execute(&pool)
            .await?;
        println!("device_tokens drop: {} rows.", drop.rows_affected());
    }

    Ok(())
}
