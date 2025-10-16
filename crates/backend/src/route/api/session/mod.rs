use axum::routing::{get, post, put};
use common::state::AppState;

pub mod http_get;
pub mod http_post;
pub mod http_put_session_id;

pub fn router() -> axum::Router<AppState> {
    axum::Router::<AppState>::new()
        .route("/", get(http_get::handler))
        .route("/", post(http_post::handler))
        .route("/{session_id}", put(http_put_session_id::handler))
}
