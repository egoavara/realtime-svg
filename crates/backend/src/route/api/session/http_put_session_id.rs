use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use common::{errors::ApiError, state::AppState, SessionInfo};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Body {
    pub args: HashMap<String, serde_json::Value>,
}

pub async fn handler(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
    Json(body): Json<Body>,
) -> Result<impl IntoResponse, ApiError> {
    let mut session_data = state
        .get_session(&session_id)
        .await?
        .ok_or(ApiError::SessionNotFound(session_id.clone()))?;
    session_data.replace_args(body.args);

    state.set_session(&session_id, &session_data, 3600).await?;

    Ok(Json(SessionInfo { session_id }))
}
