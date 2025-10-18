use axum::{extract::State, http::StatusCode, Json};
use common::{errors::ApiError, jwt, state::AppState, user_data::UserData};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    user_id: String,
    password: String,
    #[serde(default)]
    ttl_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    token: String,
}

pub async fn handler(
    State(state): State<AppState>,
    Json(req): Json<TokenRequest>,
) -> Result<(StatusCode, Json<TokenResponse>), ApiError> {
    if req.user_id.is_empty() {
        return Err(ApiError::Unauthorized(
            "user_id cannot be empty".to_string(),
        ));
    }
    let ttl = req.ttl_seconds.unwrap_or(3600);
    let user_data = state
        .create_user_data(&req.user_id, &req.password, ttl)
        .await?;

    if !state
        .verify_user_password(&user_data, &req.password)
        .await?
    {
        return Err(ApiError::Unauthorized("Invalid credentials, 동일한 사용자명이 다른 사용자에 의해 점유되었을 수 있습니다.".to_string()));
    }

    let token = jwt::create_token(
        &req.user_id,
        state.share().get_encoding_key(state.redis_client()).await?,
        ttl,
    )?;

    Ok((StatusCode::OK, Json(TokenResponse { token })))
}
