use std::collections::HashMap;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use common::{errors::ApiError, session_data::SessionData, state::AppState, SessionInfo};
use redis::AsyncTypedCommands;
use serde::{Deserialize, Serialize};

/// 세션 생성 요청 페이로드.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Body {
    pub session_id: String,
    pub template: String,
    pub args: Option<HashMap<String, serde_json::Value>>,
    pub expire: Option<String>,
}

pub async fn handler(
    State(state): State<AppState>,
    Json(body): Json<Body>,
) -> Result<impl IntoResponse, ApiError> {
    let session_id = body.session_id.trim();
    if session_id.is_empty() {
        return Err(ApiError::InvalidSessionId);
    }
    let session_id = session_id.to_string();
    let expire = body.expire.clone().unwrap_or("1d".to_string());
    let ttl = humantime::parse_duration(&expire)?;

    let mut conn = state.connection_redis().await?;
    if conn.exists(&session_id).await? {
        return Err(ApiError::SessionExists(session_id));
    }
    let session = SessionData::new(body.template.clone(), body.args.clone().unwrap_or_default());

    state
        .set_session(&session_id, &session, ttl.as_secs())
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(SessionInfo {
            session_id: session_id.clone(),
        }),
    ))
}
