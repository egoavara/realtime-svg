use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};

use crate::{errors::ApiError, jwt, state::AppState};

/// Axum extractor for JWT-authenticated users
///
/// Extracts and validates JWT from `Authorization: Bearer <token>` header.
/// Returns the authenticated user_id from the token's `sub` claim.
///
/// # Usage in Axum Handlers
/// ```ignore
/// async fn my_handler(
///     AuthenticatedUser(user_id): AuthenticatedUser,
/// ) -> Result<StatusCode, ApiError> {
///     // user_id is guaranteed to be authenticated
///     Ok(StatusCode::OK)
/// }
/// ```
///
/// # Authentication Flow
/// 1. Extract `Authorization` header from request
/// 2. Parse `Bearer <token>` format
/// 3. Verify JWT signature using RSA public key
/// 4. Validate expiration and issuer claims
/// 5. Return user_id from `sub` claim
///
/// # Error Cases
/// Returns `ApiError::Unauthorized` (401) if:
/// - Authorization header is missing
/// - Header format is invalid (not "Bearer <token>")
/// - Token signature is invalid
/// - Token has expired
/// - Token issuer doesn't match
#[derive(Debug, Clone)]
pub struct AuthenticatedUser(pub String);

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers.get("Authorization").ok_or_else(|| {
            tracing::warn!("Unauthorized: Missing Authorization header");
            ApiError::Unauthorized("Missing Authorization header".to_string())
        })?;

        let token = auth_header
            .to_str()
            .map_err(|_| {
                tracing::warn!("Unauthorized: Invalid header encoding");
                ApiError::Unauthorized("Invalid header encoding".to_string())
            })?
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                tracing::warn!(
                    "Unauthorized: Invalid Authorization format (missing Bearer prefix)"
                );
                ApiError::Unauthorized("Invalid Authorization format".to_string())
            })?;

        let app_state = AppState::from_ref(state);
        let decoding_key = app_state
            .share()
            .get_decoding_key(app_state.redis_client())
            .await?;

        let claims = jwt::verify_token(token, decoding_key)?;

        Ok(AuthenticatedUser(claims.sub))
    }
}
