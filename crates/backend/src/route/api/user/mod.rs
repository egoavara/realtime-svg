use axum::{
    routing::{get, post, put},
    Router,
};
use common::state::AppState;

mod http_get_session_id;
mod http_get_sessions;
mod http_post_session;
mod http_put_session;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{user_id}/session", post(http_post_session::handler))
        .route(
            "/{user_id}/session/{session_id}",
            put(http_put_session::handler),
        )
        .route("/{user_id}/session", get(http_get_sessions::handler))
        .route(
            "/{user_id}/session/{session_id}",
            get(http_get_session_id::handler),
        )
}
