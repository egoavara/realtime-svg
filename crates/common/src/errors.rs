use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use humantime::DurationError;
use redis::RedisError;

#[derive(Debug)]
pub enum ApiError {
    Redis(RedisError),
    Json(serde_json::Error),
    SessionExists(String),
    SessionNotFound(String),
    InvalidSessionId,
    InvalidExpire(String),
    InvalidDuration(humantime::DurationError),
    Unauthorized(String),
    Forbidden(String),
    InternalError(String),
    RedisError(String),
}

impl From<DurationError> for ApiError {
    fn from(value: DurationError) -> Self {
        ApiError::InvalidDuration(value)
    }
}

impl From<RedisError> for ApiError {
    fn from(value: RedisError) -> Self {
        ApiError::Redis(value)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(value: serde_json::Error) -> Self {
        ApiError::Json(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Redis(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Redis 오류: {err}"),
            ),
            ApiError::Json(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("직렬화 오류: {err}"),
            ),
            ApiError::SessionExists(id) => (
                StatusCode::CONFLICT,
                format!("이미 존재하는 세션 ID 입니다: {id}"),
            ),
            ApiError::SessionNotFound(id) => (
                StatusCode::NOT_FOUND,
                format!("세션을 찾을 수 없습니다: {id}"),
            ),
            ApiError::InvalidSessionId => (
                StatusCode::BAD_REQUEST,
                "세션 ID는 비어 있을 수 없습니다".to_string(),
            ),
            ApiError::InvalidExpire(message) => (StatusCode::BAD_REQUEST, message),
            ApiError::InvalidDuration(message) => (StatusCode::BAD_REQUEST, message.to_string()),
            ApiError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message),
            ApiError::Forbidden(message) => (StatusCode::FORBIDDEN, message),
            ApiError::InternalError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            ApiError::RedisError(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Redis 오류: {}", message),
            ),
        };
        let body = serde_json::json!({ "error": message });
        (status, Json(body)).into_response()
    }
}
