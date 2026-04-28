use actix_multipart::Multipart;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use futures_util::StreamExt as _;
use std::io::Write;
use uuid::Uuid;

use crate::models::Claims;
use crate::services::AppConfig;

/// POST /api/upload — Upload a file (image, voice note, etc.). Multipart form-data, max 10MB.
/// Returns { "url": "/uploads/filename" }
#[utoipa::path(
    post,
    path = "/api/upload",
    tag = "Upload",
    security(("bearer_auth" = [])),
    request_body(content = String, description = "Multipart form-data with the file. Allowed: jpg/jpeg/png/gif/webp/mp3/m4a/aac/ogg/wav/opus", content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "{ url }"),
        (status = 400, description = "No file, disallowed type, or > 10MB"),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn upload_file(
    req: HttpRequest,
    mut payload: Multipart,
    config: web::Data<AppConfig>,
) -> HttpResponse {
    // Auth check
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let mut file_url = String::new();

    while let Some(Ok(mut field)) = payload.next().await {
        let original_name = field
            .content_disposition()
            .and_then(|cd| cd.get_filename().map(String::from))
            .unwrap_or_else(|| "file".to_string());

        // Determine extension
        let ext = original_name
            .rsplit('.')
            .next()
            .unwrap_or("bin")
            .to_lowercase();

        // Validate allowed extensions
        let allowed = [
            "jpg", "jpeg", "png", "gif", "webp", "mp3", "m4a", "aac", "ogg", "wav", "opus",
        ];
        if !allowed.contains(&ext.as_str()) {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "File type not allowed"}));
        }

        // Generate unique filename
        let filename = format!("{}_{}.{}", claims.sub, Uuid::new_v4(), ext);
        let filepath = format!("uploads/{}", filename);

        // Write file
        let mut file = match std::fs::File::create(&filepath) {
            Ok(f) => f,
            Err(e) => {
                log::error!("Failed to create file: {}", e);
                return HttpResponse::InternalServerError()
                    .json(serde_json::json!({"error": "Failed to save file"}));
            }
        };

        let mut total_size: usize = 0;
        const MAX_SIZE: usize = 10 * 1024 * 1024; // 10MB

        while let Some(Ok(chunk)) = field.next().await {
            total_size += chunk.len();
            if total_size > MAX_SIZE {
                let _ = std::fs::remove_file(&filepath);
                return HttpResponse::BadRequest()
                    .json(serde_json::json!({"error": "File too large (max 10MB)"}));
            }
            if file.write_all(&chunk).is_err() {
                let _ = std::fs::remove_file(&filepath);
                return HttpResponse::InternalServerError()
                    .json(serde_json::json!({"error": "Failed to write file"}));
            }
        }

        // Build URL — in prod use APP_URL, locally use relative path
        let base_url = &config.app_url;
        file_url = if base_url.is_empty()
            || base_url.contains("localhost")
            || base_url.contains("127.0.0.1")
        {
            format!("/uploads/{}", filename)
        } else {
            format!("{}/uploads/{}", base_url, filename)
        };

        break; // Only handle one file
    }

    if file_url.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "No file provided"}));
    }

    HttpResponse::Ok().json(serde_json::json!({"url": file_url}))
}
