use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use common::{errors::ApiError, state::AppState, whoami::ExtractWhoAmI};
use tracing::info;

use crate::stream_sender::{StreamSender, StreamSenderConfigParams, StreamSenderRequest};

pub async fn handler(
    Path((user_id, session_id)): Path<(String, String)>,
    ExtractWhoAmI(whoami): ExtractWhoAmI,
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<StreamSenderConfigParams>,
) -> Result<impl IntoResponse, ApiError> {
    info!(
        "User stream access: user_id={}, session_id={}",
        user_id, session_id
    );

    let session = state
        .get_user_session(&user_id, &session_id)
        .await?
        .ok_or(ApiError::SessionNotFound(session_id.clone()))?;
    let initial_frame = session.current_frame();

    StreamSender::from_params(
        &state,
        query,
        whoami,
        &headers,
        StreamSenderRequest {
            session_log_id: format!("{}:{}", user_id, session_id),
            redirect_path: format!("/session/{}", session_id),
            initial_frame,
            pubsub_channel: format!("user:{}:session:{}", user_id, session_id),
        },
    )
    .await
}
