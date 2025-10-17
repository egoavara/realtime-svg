mod helpers;

use axum::{body::Body, http::{Request, StatusCode}};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_create_user_session_with_valid_jwt() {
    let app = helpers::create_test_app().await;
    let user_id = helpers::unique_user_id("creator");
    let session_id = helpers::unique_session_id("session");

    let token = helpers::issue_token(app.clone(), &user_id, None).await;

    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/user/{}/session", user_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "session_id": session_id,
                "template": "<svg width=\"200\" height=\"100\"><text>{{msg}}</text></svg>",
                "args": {"msg": "Hello"}
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body_json["user_id"], user_id);
    assert_eq!(body_json["session_id"], session_id);
}

#[tokio::test]
async fn test_access_user_stream_without_auth() {
    let app = helpers::create_test_app().await;
    let user_id = helpers::unique_user_id("stream_user");
    let session_id = helpers::unique_session_id("stream_session");

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
                "args": {}
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let stream_request = Request::builder()
        .method("GET")
        .uri(format!("/stream/{}/{}", user_id, session_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(stream_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_reject_session_creation_with_missing_jwt() {
    let app = helpers::create_test_app().await;
    let user_id = helpers::unique_user_id("no_jwt");
    let session_id = helpers::unique_session_id("session");

    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/user/{}/session", user_id))
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "session_id": session_id,
                "template": "<svg></svg>",
                "args": {}
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_reject_session_creation_with_user_id_mismatch() {
    let app = helpers::create_test_app().await;
    let user_alice = helpers::unique_user_id("alice");
    let user_bob = helpers::unique_user_id("bob");
    let session_id = helpers::unique_session_id("session");

    let token = helpers::issue_token(app.clone(), &user_alice, None).await;

    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/user/{}/session", user_bob))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "session_id": session_id,
                "template": "<svg></svg>",
                "args": {}
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
