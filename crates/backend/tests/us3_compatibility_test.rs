use axum::{body::Body, http::{Request, StatusCode}};
use serde_json::json;
use tower::ServiceExt;

mod helpers;

#[tokio::test]
async fn test_create_public_session_without_jwt() {
    let app = helpers::create_test_app().await;
    let session_id = helpers::unique_session_id("public");

    let request = Request::builder()
        .method("POST")
        .uri("/api/session")
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
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_update_public_session_without_jwt() {
    let app = helpers::create_test_app().await;
    let session_id = helpers::unique_session_id("public_update");

    let create_request = Request::builder()
        .method("POST")
        .uri("/api/session")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "session_id": session_id,
                "template": "<svg></svg>",
                "args": {"value": "initial"}
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let update_request = Request::builder()
        .method("PUT")
        .uri(format!("/api/session/{}", session_id))
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "args": {"value": "updated"}
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(update_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_access_public_stream_without_jwt() {
    let app = helpers::create_test_app().await;
    let session_id = helpers::unique_session_id("public_stream");

    let create_request = Request::builder()
        .method("POST")
        .uri("/api/session")
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

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let stream_request = Request::builder()
        .method("GET")
        .uri(format!("/stream/{}", session_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(stream_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_public_and_user_sessions_coexist() {
    let app = helpers::create_test_app().await;
    let public_session_id = helpers::unique_session_id("public");
    let user_id = helpers::unique_user_id("user");
    let user_session_id = helpers::unique_session_id("user");

    let public_request = Request::builder()
        .method("POST")
        .uri("/api/session")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "session_id": public_session_id,
                "template": "<svg>Public</svg>",
                "args": {}
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(public_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let token = helpers::issue_token(app.clone(), &user_id, None).await;

    let user_request = Request::builder()
        .method("POST")
        .uri(format!("/api/user/{}/session", user_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "session_id": user_session_id,
                "template": "<svg>User</svg>",
                "args": {}
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(user_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}
