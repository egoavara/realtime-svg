use axum::{routing::post, Router};
use common::state::AppState;

mod http_post_token;

pub fn router() -> Router<AppState> {
    Router::new().route("/token", post(http_post_token::handler))
}
