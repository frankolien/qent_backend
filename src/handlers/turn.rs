use actix_web::{HttpResponse, web};
use std::env;

/// GET /turn-credentials — Returns short-lived ICE servers for WebRTC.
///
/// Calls Metered's API server-side using a secret API key held in the
/// `METERED_API_KEY` env var. Keeps the secret out of the mobile binary
/// while still letting the client get fresh, time-limited TURN
/// credentials per call. Falls back to public STUN servers if Metered
/// is unreachable, so calls between users on the same LAN still work.
///
/// Response shape matches what `flutter_webrtc`'s `RTCConfiguration.iceServers`
/// expects: a JSON array of `{urls, username?, credential?}`.
pub async fn get_turn_credentials() -> HttpResponse {
    let domain = env::var("METERED_TURN_DOMAIN")
        .unwrap_or_else(|_| "qentonline.metered.live".to_string());
    let api_key = match env::var("METERED_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            log::warn!(
                "METERED_API_KEY not set — returning STUN-only fallback. Cross-network calls will fail behind symmetric NATs."
            );
            return HttpResponse::Ok().json(stun_fallback());
        }
    };

    let url = format!(
        "https://{}/api/v1/turn/credentials?apiKey={}",
        domain, api_key
    );

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::error!("turn-credentials: client build failed: {e}");
            return HttpResponse::Ok().json(stun_fallback());
        }
    };

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(json) => HttpResponse::Ok().json(json),
                Err(e) => {
                    log::error!("turn-credentials: parse failed: {e}");
                    HttpResponse::Ok().json(stun_fallback())
                }
            }
        }
        Ok(resp) => {
            log::error!("turn-credentials: Metered HTTP {}", resp.status());
            HttpResponse::Ok().json(stun_fallback())
        }
        Err(e) => {
            log::error!("turn-credentials: request failed: {e}");
            HttpResponse::Ok().json(stun_fallback())
        }
    }
}

fn stun_fallback() -> serde_json::Value {
    serde_json::json!([
        {"urls": "stun:stun.l.google.com:19302"},
        {"urls": "stun:stun1.l.google.com:19302"},
    ])
}
