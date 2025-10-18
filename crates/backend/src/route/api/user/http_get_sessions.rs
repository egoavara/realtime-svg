use axum::{
    extract::{Path, State},
    Json,
};
use common::{
    auth::AuthenticatedUser, errors::ApiError, state::AppState, ListResponse, SessionInfo,
};
use serde::Serialize;

pub async fn handler(
    State(state): State<AppState>,
    AuthenticatedUser(user_id_from_token): AuthenticatedUser,
    Path(user_id): Path<String>,
) -> Result<Json<ListResponse<SessionInfo>>, ApiError> {
    if user_id_from_token != user_id {
        tracing::warn!(
            "User {} attempted to list sessions of user {}",
            user_id_from_token,
            user_id
        );
        return Err(ApiError::Forbidden(format!(
            "User {} cannot list sessions of user {}",
            user_id_from_token, user_id
        )));
    }

    let session_ids = state.list_user_sessions(&user_id).await?;

    let sessions = session_ids
        .into_iter()
        .map(|session_id| SessionInfo { session_id })
        .collect();

    Ok(Json(ListResponse { items: sessions }))
}
