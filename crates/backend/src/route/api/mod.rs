use axum::Router;
use common::state::AppState;

pub mod auth;
pub mod session;
pub mod user;

pub fn router() -> Router<AppState> {
    axum::Router::new()
        .nest("/auth", auth::router())
        .nest("/session", session::router())
        .nest("/user", user::router())
}
