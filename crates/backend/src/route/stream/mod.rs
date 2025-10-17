use axum::{routing::get, Router};
use common::state::AppState;

pub mod http_get;
pub mod http_get_user_stream;

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/{session_id}", get(http_get::handler))
        .route(
            "/{user_id}/{session_id}",
            get(http_get_user_stream::handler),
        )
}
