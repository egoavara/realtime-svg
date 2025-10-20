use std::convert::Infallible;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, Response, StatusCode},
    response::{IntoResponse, Redirect},
};
use bytes::Bytes;
use common::{
    browser_engine::WellKnownBrowserEngine, errors::ApiError, state::AppState,
    whoami::ExtractWhoAmI, SvgFrame,
};
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tokio_stream::{once, wrappers::ReceiverStream, StreamExt};
use tracing::{error, warn};

#[derive(serde::Deserialize)]
pub struct QueryParams {
    double_frame: Option<bool>,
    as_bot: Option<bool>,
    keep_alive: Option<u64>,
}

pub async fn handler(
    Path(session_id): Path<String>,
    ExtractWhoAmI(whoami): ExtractWhoAmI,
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<QueryParams>,
) -> Result<impl IntoResponse, ApiError> {
    if query.as_bot != Some(true) && whoami == common::whoami::WhoAmI::Human {
        return Ok(Redirect::temporary(&format!("/session/{}", session_id)).into_response());
    }

    let session = state
        .get_session(&session_id)
        .await?
        .ok_or(ApiError::SessionNotFound(session_id.clone()))?;

    let initial_frame = session.current_frame();

    let mut pubsub = state.pubsub().await?;
    pubsub.subscribe(&session_id).await?;

    let (tx, rx) = mpsc::channel::<SvgFrame>(16);
    let tx_clone = tx.clone();
    let session_for_log = session_id.clone();
    let keep_alive_interval = query.keep_alive;

    tokio::spawn(async move {
        let mut pubsub = pubsub;
        let mut on_message = pubsub.on_message();
        let mut last_frame: Option<SvgFrame> = None;
        let mut keep_alive_timer = keep_alive_interval.map(|secs| interval(Duration::from_secs(secs)));

        loop {
            tokio::select! {
                msg = on_message.next() => {
                    match msg {
                        Some(msg) => {
                            let payload: String = match msg.get_payload() {
                                Ok(payload) => payload,
                                Err(err) => {
                                    error!(session_id = %session_for_log, %err, "Redis 메시지 페이로드를 읽지 못했습니다");
                                    continue;
                                }
                            };

                            match serde_json::from_str::<SvgFrame>(&payload) {
                                Ok(frame) => {
                                    last_frame = Some(frame.clone());
                                    if tx_clone.send(frame).await.is_err() {
                                        break;
                                    }
                                }
                                Err(err) => {
                                    error!(session_id = %session_for_log, %err, "Redis 메시지를 SVG 프레임으로 역직렬화하지 못했습니다");
                                }
                            }
                        }
                        None => break,
                    }
                }
                _ = async {
                    if let Some(ref mut timer) = keep_alive_timer {
                        timer.tick().await;
                    } else {
                        std::future::pending::<()>().await;
                    }
                } => {
                    if let Some(ref frame) = last_frame {
                        if tx_clone.send(frame.clone()).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });
    drop(tx);

    let browser = WellKnownBrowserEngine::from_header(&headers);
    let should_double = query
        .double_frame
        .unwrap_or_else(|| browser.is_x_multipart_replace_double_frame());

    let start_boundary = once(Ok::<Bytes, Infallible>(Bytes::from_static(b"--frame\r\n")));
    let initial_part = once(Ok::<Bytes, Infallible>(Bytes::from(encode_stream_frame(
        &initial_frame,
        should_double,
    ))));
    let updates = ReceiverStream::new(rx).map(move |frame| {
        let bytes = encode_stream_frame(&frame, should_double);
        Ok::<Bytes, Infallible>(Bytes::from(bytes))
    });

    let stream = start_boundary.chain(initial_part).chain(updates);
    let body = Body::from_stream(stream);

    let mut response = Response::new(body);
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("multipart/x-mixed-replace; boundary=frame"),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    Ok(response)
}

fn encode_stream_frame(frame: &SvgFrame, duplicate: bool) -> Vec<u8> {
    if duplicate {
        let mut bytes = encode_multipart(frame);
        bytes.extend_from_slice(&encode_multipart(frame));
        bytes
    } else {
        encode_multipart(frame)
    }
}

fn encode_multipart(frame: &SvgFrame) -> Vec<u8> {
    let mut output = Vec::with_capacity(frame.content.len() + 2);
    output.extend_from_slice(b"Content-Type: image/svg+xml\r\n");
    output.extend_from_slice(format!("Content-Length: {}\r\n", frame.content.len()).as_bytes());
    output.extend_from_slice(b"\r\n");
    output.extend_from_slice(frame.content.as_bytes());
    output.extend_from_slice(b"\r\n");
    output.extend_from_slice(b"--frame\r\n");
    output
}
