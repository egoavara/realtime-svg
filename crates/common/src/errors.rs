use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use humantime::DurationError;
use redis::RedisError;

#[derive(Debug)]
pub enum ApiError {
    Argon2(argon2::Error),
    Argon2PasswordHash(argon2::password_hash::Error),
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
    Unexpected(String),
}

macro_rules! wrap_error {
    ($err_type:ty, $variant:ident) => {
        impl From<$err_type> for ApiError {
            fn from(value: $err_type) -> Self {
                ApiError::$variant(value)
            }
        }
    };
}

wrap_error!(argon2::Error, Argon2);
wrap_error!(argon2::password_hash::Error, Argon2PasswordHash);
wrap_error!(RedisError, Redis);
wrap_error!(serde_json::Error, Json);
wrap_error!(DurationError, InvalidDuration);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Argon2(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("암호화 오류: {err}"),
            ),
            ApiError::Argon2PasswordHash(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("암호화 해시 오류: {err}"),
            ),
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
            ApiError::Unexpected(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("예상하지 못한 오류입니다: {}", message),
            ),
        };
        let body = serde_json::json!({ "error": message });
        (status, Json(body)).into_response()
    }
}
