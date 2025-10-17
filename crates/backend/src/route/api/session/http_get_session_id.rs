use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use common::{errors::ApiError, state::AppState, SessionDetailInfo};

pub async fn handler(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let session_data = state
        .get_session(&session_id)
        .await?
        .ok_or(ApiError::SessionNotFound(session_id.clone()))?;

    Ok(Json(SessionDetailInfo {
        session_id,
        template: session_data.template,
        args: session_data.args,
    }))
}
