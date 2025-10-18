mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_list_user_sessions_returns_only_owners_sessions() {
    let app = helpers::create_test_app().await;
    let user_id = helpers::unique_user_id("lister");

    let token = helpers::issue_token(app.clone(), &user_id, None).await;

    for i in 1..=3 {
        let session_id = format!("session_{}_{}", i, uuid::Uuid::new_v4());
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
    }

    let list_request = Request::builder()
        .method("GET")
        .uri(format!("/api/user/{}/sessions", user_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(list_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(body_json["sessions"].is_array());
    let sessions = body_json["sessions"].as_array().unwrap();
    assert_eq!(sessions.len(), 3);
}

#[tokio::test]
async fn test_empty_list_for_user_with_no_sessions() {
    let app = helpers::create_test_app().await;
    let user_id = helpers::unique_user_id("no_sessions");

    let token = helpers::issue_token(app.clone(), &user_id, None).await;

    let list_request = Request::builder()
        .method("GET")
        .uri(format!("/api/user/{}/sessions", user_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(list_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(body_json["sessions"].is_array());
    let sessions = body_json["sessions"].as_array().unwrap();
    assert_eq!(sessions.len(), 0);
}

#[tokio::test]
async fn test_list_excludes_other_users_sessions() {
    let app = helpers::create_test_app().await;
    let user_alice = helpers::unique_user_id("alice");
    let user_bob = helpers::unique_user_id("bob");

    let alice_token = helpers::issue_token(app.clone(), &user_alice, None).await;
    let bob_token = helpers::issue_token(app.clone(), &user_bob, None).await;

    let alice_session = helpers::unique_session_id("alice_session");
    let create_alice_request = Request::builder()
        .method("POST")
        .uri(format!("/api/user/{}/session", user_alice))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", alice_token))
        .body(Body::from(
            json!({
                "session_id": alice_session,
                "template": "<svg></svg>",
                "args": {}
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_alice_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let bob_session = helpers::unique_session_id("bob_session");
    let create_bob_request = Request::builder()
        .method("POST")
        .uri(format!("/api/user/{}/session", user_bob))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", bob_token))
        .body(Body::from(
            json!({
                "session_id": bob_session,
                "template": "<svg></svg>",
                "args": {}
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_bob_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let list_alice_request = Request::builder()
        .method("GET")
        .uri(format!("/api/user/{}/sessions", user_alice))
        .header("authorization", format!("Bearer {}", alice_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(list_alice_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let sessions = body_json["sessions"].as_array().unwrap();
    assert_eq!(sessions.len(), 1);

    let session_ids: Vec<String> = sessions
        .iter()
        .map(|s| s["session_id"].as_str().unwrap().to_string())
        .collect();
    assert!(session_ids.contains(&alice_session));
    assert!(!session_ids.contains(&bob_session));
}
