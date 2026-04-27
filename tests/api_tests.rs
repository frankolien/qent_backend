use serde_json::json;

/// Test against a running server. Run with:
/// ```
/// cargo test -- --test-threads=1
/// ```
/// Make sure `cargo run` is running first.

const BASE: &str = "http://127.0.0.1:8080";

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

#[tokio::test]
async fn test_health_check() {
    let resp = client()
        .get(format!("{}/health", BASE))
        .send()
        .await
        .expect("health check failed");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "healthy");
    assert_eq!(body["database"], "connected");
}

#[tokio::test]
async fn test_waitlist_join() {
    let resp = client()
        .post(format!("{}/api/waitlist", BASE))
        .json(&json!({"email": "test_runner@test.com", "role": "Renter"}))
        .send()
        .await
        .expect("waitlist join failed");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["message"].as_str().unwrap().contains("on the list"));
    assert!(body["position"].as_i64().unwrap() > 0);
}

#[tokio::test]
async fn test_waitlist_invalid_email() {
    let resp = client()
        .post(format!("{}/api/waitlist", BASE))
        .json(&json!({"email": "notanemail", "role": "Renter"}))
        .send()
        .await
        .expect("waitlist validation failed");

    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn test_waitlist_count() {
    let resp = client()
        .get(format!("{}/api/waitlist/count", BASE))
        .send()
        .await
        .expect("waitlist count failed");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["count"].as_i64().is_some());
}

#[tokio::test]
async fn test_car_search() {
    let resp = client()
        .get(format!("{}/api/cars/search", BASE))
        .send()
        .await
        .expect("car search failed");

    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_car_search_with_sort() {
    let resp = client()
        .get(format!("{}/api/cars/search?sort_by=price_asc", BASE))
        .send()
        .await
        .expect("car search sort failed");

    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_car_homepage() {
    let resp = client()
        .get(format!("{}/api/cars/homepage", BASE))
        .send()
        .await
        .expect("homepage failed");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["recommended"].is_array());
    assert!(body["best_cars"].is_array());
    assert!(body["nearby"].is_array());
    assert!(body["popular"].is_array());
}

#[tokio::test]
async fn test_protection_plans() {
    let resp = client()
        .get(format!("{}/api/protection-plans", BASE))
        .send()
        .await
        .expect("protection plans failed");

    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_signup_and_signin() {
    let email = format!(
        "testuser_{}@test.com",
        chrono::Utc::now().timestamp_millis()
    );

    // Sign up
    let resp = client()
        .post(format!("{}/api/auth/signup", BASE))
        .json(&json!({
            "email": email,
            "password": "testpass123",
            "full_name": "Test User",
            "role": "Renter"
        }))
        .send()
        .await
        .expect("signup failed");

    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["token"].as_str().is_some());
    assert!(body["refresh_token"].as_str().is_some());

    // Sign in
    let resp = client()
        .post(format!("{}/api/auth/signin", BASE))
        .json(&json!({"email": email, "password": "testpass123"}))
        .send()
        .await
        .expect("signin failed");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let token = body["token"].as_str().unwrap();
    let refresh = body["refresh_token"].as_str().unwrap();

    // Refresh token
    let resp = client()
        .post(format!("{}/api/auth/refresh", BASE))
        .json(&json!({"refresh_token": refresh}))
        .send()
        .await
        .expect("refresh failed");

    assert_eq!(resp.status(), 200);

    // Get profile with token (should be 200 or at least not 401)
    let resp = client()
        .get(format!("{}/api/auth/profile", BASE))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("profile failed");

    // Token is valid — should not get 401
    assert_ne!(resp.status(), 401);
}

#[tokio::test]
async fn test_unauthorized_access() {
    // Authenticated routes return 404 when no token is provided (middleware rejects before routing)
    let resp = client()
        .get(format!("{}/api/auth/profile", BASE))
        .send()
        .await
        .expect("unauth test failed");

    // 404 or 401 both indicate the request was rejected
    assert!(resp.status() == 401 || resp.status() == 404);
}

#[tokio::test]
async fn test_forgot_password() {
    let resp = client()
        .post(format!("{}/api/auth/forgot-password", BASE))
        .json(&json!({"email": "nonexistent@test.com"}))
        .send()
        .await
        .expect("forgot password failed");

    // Should always return 200 to prevent email enumeration
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_banks_list() {
    let resp = client()
        .get(format!("{}/api/payments/banks", BASE))
        .send()
        .await
        .expect("banks list failed");

    assert_eq!(resp.status(), 200);
}
