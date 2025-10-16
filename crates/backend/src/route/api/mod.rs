use axum::Router;
use common::state::AppState;

pub mod session;

pub fn router() -> Router<AppState> {
    axum::Router::new().nest("/session", session::router())
}
