use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use common::{
    auth::AuthenticatedUser, errors::ApiError, session_data::SessionData, state::AppState,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    session_id: String,
    template: String,
    args: std::collections::HashMap<String, serde_json::Value>,
    #[serde(default = "default_ttl")]
    ttl_seconds: u64,
}

fn default_ttl() -> u64 {
    3600
}

#[derive(Debug, Serialize)]
pub struct CreateSessionResponse {
    user_id: String,
    session_id: String,
}

pub async fn handler(
    State(state): State<AppState>,
    AuthenticatedUser(user_id_from_token): AuthenticatedUser,
    Path(user_id): Path<String>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<CreateSessionResponse>), ApiError> {
    if user_id_from_token != user_id {
        tracing::warn!(
            "User {} attempted to create session for user {}",
            user_id_from_token,
            user_id
        );
        return Err(ApiError::Forbidden(format!(
            "User {} cannot create sessions for user {}",
            user_id_from_token, user_id
        )));
    }

    if req.session_id.is_empty() {
        return Err(ApiError::InvalidSessionId);
    }

    let session = SessionData::new_with_owner(req.template, req.args, user_id.clone());

    state
        .set_user_session(&user_id, &req.session_id, &session, req.ttl_seconds)
        .await?;

    tracing::info!(
        "Created user session: user={}, session_id={}, owner={}",
        user_id,
        req.session_id,
        user_id
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateSessionResponse {
            user_id,
            session_id: req.session_id,
        }),
    ))
}
