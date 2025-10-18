use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use common::{auth::AuthenticatedUser, errors::ApiError, state::AppState, SessionDetailInfo};
use serde::Serialize;

pub async fn handler(
    Path((user_id, session_id)): Path<(String, String)>,
    AuthenticatedUser(user_id_from_token): AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    if user_id_from_token != user_id {
        tracing::warn!(
            "User {} attempted to access session of user {}",
            user_id_from_token,
            user_id
        );
        return Err(ApiError::Forbidden(format!(
            "User {} cannot access sessions of user {}",
            user_id_from_token, user_id
        )));
    }
    let session_data = state
        .get_user_session(&user_id, &session_id)
        .await?
        .ok_or(ApiError::SessionNotFound(session_id.clone()))?;

    Ok(Json(SessionDetailInfo {
        session_id,
        template: session_data.template,
        args: session_data.args,
    }))
}
