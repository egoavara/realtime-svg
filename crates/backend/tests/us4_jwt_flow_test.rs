mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_token_issuance_flow() {
    let app = helpers::create_test_app().await;
    let user_id = helpers::unique_user_id("test");

    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/token")
        .header("content-type", "application/json")
        .body(Body::from(json!({"user_id": user_id}).to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(body_json["token"].is_string());
    let token = body_json["token"].as_str().unwrap();
    assert!(!token.is_empty());
    assert!(token.contains('.'));
}

#[tokio::test]
async fn test_invalid_token_rejection() {
    let app = helpers::create_test_app().await;
    let user_id = helpers::unique_user_id("invalid");
    let session_id = helpers::unique_session_id("test");

    let invalid_token = "invalid.token.here";

    let create_session_request = Request::builder()
        .method("POST")
        .uri(format!("/api/user/{}/session", user_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", invalid_token))
        .body(Body::from(
            json!({
                "session_id": session_id,
                "template": "<svg></svg>",
                "args": {}
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(create_session_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_expired_token_rejection() {
    let app = helpers::create_test_app().await;
    let user_id = helpers::unique_user_id("expired");

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let redis_client = redis::Client::open(redis_url).unwrap();
    let expired_token = helpers::create_expired_jwt(&redis_client, &user_id).await;

    let session_id = helpers::unique_session_id("test");
    let create_session_request = Request::builder()
        .method("POST")
        .uri(format!("/api/user/{}/session", user_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", expired_token))
        .body(Body::from(
            json!({
                "session_id": session_id,
                "template": "<svg></svg>",
                "args": {}
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(create_session_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_jwks_endpoint_returns_valid_jwkset() {
    let app = helpers::create_test_app().await;

    let request = Request::builder()
        .method("GET")
        .uri("/.well-known/jwks.json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let jwks: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(jwks["keys"].is_array());
    let keys = jwks["keys"].as_array().unwrap();
    assert!(!keys.is_empty());

    let key = &keys[0];
    assert_eq!(key["kty"], "RSA");
    assert_eq!(key["alg"], "RS256");
    assert_eq!(key["use"], "sig");
    assert!(key["n"].is_string());
    assert!(key["e"].is_string());
}
