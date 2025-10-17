use axum::{extract::State, http::StatusCode, Json};
use common::{errors::ApiError, jwt, state::AppState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    user_id: String,
    #[serde(default)]
    ttl_seconds: Option<i64>,
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

    let encoding_key = state
        .jwk_cache()
        .get_encoding_key(state.redis_client())
        .await?;

    let token = jwt::create_token(&req.user_id, encoding_key, req.ttl_seconds)?;

    tracing::info!("Issued JWT token for user: {}", req.user_id);

    Ok((StatusCode::OK, Json(TokenResponse { token })))
}
