use axum::{body::Body, http::Request, Router};
use common::state::AppState;
use redis::Client;
use serde_json::json;
use std::sync::Once;

static INIT_LOGGER: Once = Once::new();

fn init_logger() {
    INIT_LOGGER.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
            )
            .with_test_writer()
            .init();
    });
}

pub async fn create_test_app() -> Router {
    init_logger();
    
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let redis_client = Client::open(redis_url).unwrap();

    common::jwk::initialize_jwk_in_redis(&redis_client)
        .await
        .unwrap();

    let state = AppState::new(redis_client);
    backend::route::router().with_state(state)
}

#[allow(dead_code)]
pub async fn issue_token(app: Router, user_id: &str, ttl_seconds: Option<i64>) -> String {
    use tower::ServiceExt;

    let mut payload = json!({"user_id": user_id});
    if let Some(ttl) = ttl_seconds {
        payload["ttl_seconds"] = json!(ttl);
    }

    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/token")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    json["token"].as_str().unwrap().to_string()
}

pub fn unique_user_id(prefix: &str) -> String {
    format!("{}_{}", prefix, uuid::Uuid::new_v4())
}

pub fn unique_session_id(prefix: &str) -> String {
    format!("{}_{}", prefix, uuid::Uuid::new_v4())
}

#[allow(dead_code)]
pub async fn create_expired_jwt(redis_client: &redis::Client, user_id: &str) -> String {
    use chrono::{Duration, Utc};
    use jsonwebtoken::{encode, Algorithm, Header};
    
    let cache = common::jwk::JwkCache::new();
    let encoding_key = cache.get_encoding_key(redis_client).await.unwrap();
    
    let now = Utc::now();
    let exp = now - Duration::hours(2);
    
    let claims = common::jwt::Claims {
        sub: user_id.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
        iss: "realtime-svg".to_string(),
    };
    
    let header = Header::new(Algorithm::RS256);
    encode(&header, &claims, encoding_key).unwrap()
}
