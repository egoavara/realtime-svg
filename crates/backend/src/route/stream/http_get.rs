use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use common::{errors::ApiError, state::AppState, whoami::ExtractWhoAmI};
use tracing::info;

use crate::stream_sender::{StreamSender, StreamSenderConfigParams, StreamSenderRequest};

pub async fn handler(
    Path(session_id): Path<String>,
    ExtractWhoAmI(whoami): ExtractWhoAmI,
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<StreamSenderConfigParams>,
) -> Result<impl IntoResponse, ApiError> {
    info!(session_id = %session_id, whoami = ?whoami, "HTTP GET 스트림 요청을 처리합니다");

    let session = state
        .get_session(&session_id)
        .await?
        .ok_or(ApiError::SessionNotFound(session_id.clone()))?;
    let initial_frame = session.current_frame();

    StreamSender::from_params(
        &state,
        query,
        whoami,
        &headers,
        StreamSenderRequest {
            session_log_id: session_id.clone(),
            redirect_path: format!("/session/{}", session_id),
            initial_frame,
            pubsub_channel: session_id,
        },
    )
    .await
}
