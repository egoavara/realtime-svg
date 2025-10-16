use axum::{routing::get, Router};
use common::state::AppState;

pub mod api;
pub mod index;
pub mod r#static;
pub mod stream;

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .nest("/api", api::router())
        .nest("/stream", stream::router())
        .route("/static/{*path}", get(r#static::handler))
        .route("/", get(index::handler))
}
