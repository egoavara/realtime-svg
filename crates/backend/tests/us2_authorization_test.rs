mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_owner_can_update_session() {
    let app = helpers::create_test_app().await;
    let user_id = helpers::unique_user_id("owner");
    let session_id = helpers::unique_session_id("session");

    let token = helpers::issue_token(app.clone(), &user_id, None).await;

    let create_request = Request::builder()
        .method("POST")
        .uri(format!("/api/user/{}/session", user_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "session_id": session_id,
                "template": "<svg></svg>",
                "args": {"key": "value1"}
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let update_request = Request::builder()
        .method("PUT")
        .uri(format!("/api/user/{}/session/{}", user_id, session_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "args": {"key": "value2"}
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(update_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_non_owner_gets_403_forbidden() {
    let app = helpers::create_test_app().await;
    let user_alice = helpers::unique_user_id("alice");
    let user_bob = helpers::unique_user_id("bob");
    let session_id = helpers::unique_session_id("session");

    let alice_token = helpers::issue_token(app.clone(), &user_alice, None).await;

    let create_request = Request::builder()
        .method("POST")
        .uri(format!("/api/user/{}/session", user_alice))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", alice_token))
        .body(Body::from(
            json!({
                "session_id": session_id,
                "template": "<svg></svg>",
                "args": {}
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let bob_token = helpers::issue_token(app.clone(), &user_bob, None).await;

    let update_request = Request::builder()
        .method("PUT")
        .uri(format!("/api/user/{}/session/{}", user_alice, session_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", bob_token))
        .body(Body::from(
            json!({
                "args": {"hacked": "true"}
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(update_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_unauthenticated_request_gets_401() {
    let app = helpers::create_test_app().await;
    let user_id = helpers::unique_user_id("user");
    let session_id = helpers::unique_session_id("session");

    let update_request = Request::builder()
        .method("PUT")
        .uri(format!("/api/user/{}/session/{}", user_id, session_id))
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "args": {}
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(update_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
