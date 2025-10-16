use axum::{routing::get, Router};
use common::state::AppState;

pub mod http_get;

pub fn router() -> Router<AppState> {
    Router::<AppState>::new().route("/{session_id}", get(http_get::handler))
}
