use axum::{routing::get, Router};
use common::state::AppState;

pub mod api;
pub mod fallback;
pub mod index;
pub mod r#static;
pub mod stream;
pub mod well_known;

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .nest("/api", api::router())
        .nest("/stream", stream::router())
        .nest("/.well-known", well_known::router())
        .route("/static/{*path}", get(r#static::handler))
        .route("/session/{session_id}", get(index::handler))
        .route("/", get(index::handler))
        .fallback(fallback::handler)
}
