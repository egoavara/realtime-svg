use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use common::{auth::AuthenticatedUser, errors::ApiError, state::AppState};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UpdateSessionRequest {
    args: std::collections::HashMap<String, serde_json::Value>,
}

pub async fn handler(
    State(state): State<AppState>,
    AuthenticatedUser(user_id_from_token): AuthenticatedUser,
    Path((user_id, session_id)): Path<(String, String)>,
    axum::Json(req): axum::Json<UpdateSessionRequest>,
) -> Result<StatusCode, ApiError> {
    if user_id_from_token != user_id {
        tracing::warn!(
            "User {} attempted to update session of user {}",
            user_id_from_token,
            user_id
        );
        return Err(ApiError::Forbidden(format!(
            "User {} cannot modify sessions of user {}",
            user_id_from_token, user_id
        )));
    }

    let mut session = state
        .get_user_session(&user_id, &session_id)
        .await?
        .ok_or_else(|| ApiError::SessionNotFound(session_id.clone()))?;

    session.replace_args(req.args);

    let ttl_seconds = 3600;
    state
        .set_user_session(&user_id, &session_id, &session, ttl_seconds)
        .await?;

    tracing::info!(
        "Updated user session: user={}, session_id={}",
        user_id,
        session_id
    );

    Ok(StatusCode::NO_CONTENT)
}
