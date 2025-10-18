use common::{jwt, state::AppState, session_data::SessionData};
use redis::Client;
use std::collections::HashMap;

async fn setup_test() -> (AppState, Client) {
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let redis_client = Client::open(redis_url).unwrap();

    common::share::initialize_redis(&redis_client)
        .await
        .unwrap();

    let state = AppState::new(redis_client.clone());
    (state, redis_client)
}

#[tokio::test]
async fn test_create_user_session_with_owner() {
    let (state, _) = setup_test().await;

    let user_id = "test_user";
    let session_id = "test_session";
    let mut args = HashMap::new();
    args.insert("test".to_string(), serde_json::json!("value"));

    let session = SessionData::new_with_owner(
        "<svg></svg>".to_string(),
        args,
        user_id.to_string(),
    );

    state
        .set_user_session(user_id, session_id, &session, 3600)
        .await
        .unwrap();

    let retrieved = state
        .get_user_session(user_id, session_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(retrieved.owner, Some(user_id.to_string()));
    assert_eq!(retrieved.template, "<svg></svg>");
}

#[tokio::test]
async fn test_list_user_sessions() {
    let (state, _) = setup_test().await;

    let user_id = "list_test_user";

    for i in 1..=3 {
        let session = SessionData::new_with_owner(
            "<svg></svg>".to_string(),
            HashMap::new(),
            user_id.to_string(),
        );
        state
            .set_user_session(user_id, &format!("session_{}", i), &session, 3600)
            .await
            .unwrap();
    }

    let sessions = state.list_user_sessions(user_id).await.unwrap();
    assert_eq!(sessions.len(), 3);
}

#[tokio::test]
async fn test_list_sessions_filters_by_user() {
    let (state, _) = setup_test().await;

    let user1 = "filter_user1";
    let user2 = "filter_user2";

    let session1 = SessionData::new_with_owner(
        "<svg></svg>".to_string(),
        HashMap::new(),
        user1.to_string(),
    );
    state
        .set_user_session(user1, "session_1", &session1, 3600)
        .await
        .unwrap();

    let session2 = SessionData::new_with_owner(
        "<svg></svg>".to_string(),
        HashMap::new(),
        user2.to_string(),
    );
    state
        .set_user_session(user2, "session_2", &session2, 3600)
        .await
        .unwrap();

    let user1_sessions = state.list_user_sessions(user1).await.unwrap();
    assert_eq!(user1_sessions.len(), 1);
    assert!(user1_sessions.contains(&"session_1".to_string()));
    assert!(!user1_sessions.contains(&"session_2".to_string()));
}

#[tokio::test]
async fn test_jwt_token_creation_and_verification() {
    let (state, _) = setup_test().await;

    let encoding_key = state
        .share()
        .get_encoding_key(state.redis_client())
        .await
        .unwrap();

    let decoding_key = state
        .share()
        .get_decoding_key(state.redis_client())
        .await
        .unwrap();

    let token = jwt::create_token("test_user", encoding_key, 3600).unwrap();
    let claims = jwt::verify_token(&token, decoding_key).unwrap();

    assert_eq!(claims.sub, "test_user");
    assert_eq!(claims.iss, "realtime-svg");
}
