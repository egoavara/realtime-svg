use axum::routing::{get, post, put};
use common::state::AppState;

pub mod http_get_session_id;
pub mod http_post;
pub mod http_put_session_id;

pub fn router() -> axum::Router<AppState> {
    axum::Router::<AppState>::new()
        .route("/", post(http_post::handler))
        .route("/{session_id}", get(http_get_session_id::handler))
        .route("/{session_id}", put(http_put_session_id::handler))
}
