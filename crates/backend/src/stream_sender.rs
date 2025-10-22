use std::convert::Infallible;

use axum::{
    body::Body,
    http::{header, HeaderMap, HeaderValue, Response, StatusCode},
    response::{IntoResponse, Redirect},
};
use bytes::Bytes;
use common::{browser_engine::WellKnownBrowserEngine, errors::ApiError, state::AppState, whoami::WhoAmI, SvgFrame};
use redis::aio::PubSub;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tokio_stream::{once, wrappers::ReceiverStream, StreamExt};
use tracing::{debug, error};

#[derive(serde::Deserialize)]
pub struct StreamSenderConfigParams {
    pub double_frame: Option<bool>,
    pub as_bot: Option<bool>,
    pub keep_alive: Option<u64>,
    pub delayed_start: Option<u64>,
}

pub struct StreamSenderConfig {
    pub session_log_id: String,
    pub initial_frame: SvgFrame,
    pub should_double: bool,
    pub keep_alive_ms: u64,
    pub delayed_start_ms: u64,
}

pub struct StreamSenderRequest {
    pub session_log_id: String,
    pub redirect_path: String,
    pub initial_frame: SvgFrame,
    pub pubsub_channel: String,
}

pub struct StreamSender {
    config: StreamSenderConfig,
}

impl StreamSender {
    pub fn new(config: StreamSenderConfig) -> Self {
        Self { config }
    }

    pub async fn from_params(
        state: &AppState,
        params: StreamSenderConfigParams,
        whoami: WhoAmI,
        headers: &HeaderMap,
        request: StreamSenderRequest,
    ) -> Result<impl IntoResponse, ApiError> {
        if params.as_bot != Some(true) && whoami == WhoAmI::Human {
            return Ok(Redirect::temporary(&request.redirect_path).into_response());
        }

        let mut pubsub = state.pubsub().await?;
        pubsub.subscribe(&request.pubsub_channel).await?;

        let keep_alive_interval = params.keep_alive.unwrap_or(30000);
        let delayed_start = params.delayed_start.unwrap_or(0);

        let browser = WellKnownBrowserEngine::from_header(headers);
        let should_double = params
            .double_frame
            .unwrap_or_else(|| browser.is_x_multipart_replace_double_frame());

        let sender = StreamSender::new(StreamSenderConfig {
            session_log_id: request.session_log_id,
            initial_frame: request.initial_frame,
            should_double,
            keep_alive_ms: keep_alive_interval,
            delayed_start_ms: delayed_start,
        });

        let response = sender.start_and_build_response(pubsub).await;
        Ok(response)
    }

    pub async fn start_and_build_response(
        self,
        mut pubsub: PubSub,
    ) -> Response<Body> {
        let (tx, rx) = mpsc::channel::<SvgFrame>(16);
        let tx_clone = tx.clone();

        let session_log_id = self.config.session_log_id.clone();
        let initial_frame = self.config.initial_frame.clone();
        let keep_alive_interval = self.config.keep_alive_ms;
        let delayed_start = self.config.delayed_start_ms;

        let start_boundary = once(Ok::<Bytes, Infallible>(Bytes::from_static(b"--frame\r\n")));
        let initial_part = once(Ok::<Bytes, Infallible>(Bytes::from(encode_stream_frame(
            &self.config.initial_frame,
            self.config.should_double,
        ))));
        let should_double = self.config.should_double;
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

        tokio::spawn(async move {
            let mut pubsub_stream = pubsub.on_message();
            let mut last_frame: SvgFrame = initial_frame;
            let mut keep_alive_timer = interval(Duration::from_millis(keep_alive_interval));

            tokio::time::sleep(Duration::from_millis(delayed_start)).await;

            loop {
                tokio::select! {
                    msg = pubsub_stream.next() => {
                        match msg {
                            Some(msg) => {
                                let payload: String = match msg.get_payload() {
                                    Ok(payload) => payload,
                                    Err(err) => {
                                        error!(session = %session_log_id, %err, "Redis 메시지 페이로드를 읽지 못했습니다");
                                        continue;
                                    }
                                };

                                match serde_json::from_str::<SvgFrame>(&payload) {
                                    Ok(frame) => {
                                        last_frame = frame.clone();
                                        if tx_clone.send(frame).await.is_err() {
                                            break;
                                        }
                                    }
                                    Err(err) => {
                                        error!(session = %session_log_id, %err, "Redis 메시지를 SVG 프레임으로 역직렬화하지 못했습니다");
                                    }
                                }
                            }
                            None => break,
                        }
                    }
                    _ = keep_alive_timer.tick() => {
                        debug!(frame_number = %last_frame.timestamp, session = %session_log_id, "Keep-alive 프레임을 다시 전송합니다");
                        if tx_clone.send(last_frame.clone()).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });
        drop(tx);

        response
    }
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
