use axum::{routing::get, Router};
use common::state::AppState;

pub mod http_get_jwks_json;

pub fn router() -> Router<AppState> {
    Router::new().route("/jwks.json", get(http_get_jwks_json::handler))
}
