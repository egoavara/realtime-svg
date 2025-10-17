use anyhow::Context;
use axum::Router;
use common::state::AppState;
use redis::Client;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

pub mod assets;
pub mod route;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let redis_client = Client::open(redis_url.clone())
        .with_context(|| format!("Redis에 연결할 수 없습니다: {redis_url}"))?;

    let state = AppState::new(redis_client.clone());

    if let Err(e) = common::jwk::initialize_jwk_in_redis(&redis_client).await {
        return Err(anyhow::anyhow!(
            "Failed to initialize JWK in Redis: {:?}",
            e
        ));
    }
    info!("JWK initialized successfully");

    let app = Router::<AppState>::new()
        .merge(route::router())
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    info!("서버 시작: http://127.0.0.1:3000");
    axum::serve(
        tokio::net::TcpListener::bind(("127.0.0.1", 3000)).await?,
        app,
    )
    .await?;
    Ok(())
}

fn init_tracing() {
    if tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .is_err()
    {
        // 이미 초기화된 경우 무시.
    }
}
