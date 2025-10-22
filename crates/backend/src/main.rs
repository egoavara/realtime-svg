use std::str::FromStr;

use anyhow::Context;
use axum::Router;
use common::config::Config;
use common::state::AppState;
use redis::Client;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

pub mod assets;
pub mod route;
pub mod stream_sender;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let config = Config::load().context("설정 로딩 실패")?;
    config.validate().context("설정 검증 실패")?;
    init_tracing(&config);

    info!("서버 설정: {:?}", config);

    let redis_client = Client::open(config.redis_url.clone())
        .with_context(|| format!("Redis에 연결할 수 없습니다: {}", config.redis_url))?;

    let state = AppState::new(redis_client.clone());

    if let Err(e) = common::share::initialize_redis(&redis_client).await {
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

    let bind_addr = format!("{}:{}", config.host, config.port);
    info!("서버 시작: http://{}", bind_addr);
    axum::serve(tokio::net::TcpListener::bind(&bind_addr).await?, app).await?;
    Ok(())
}

fn init_tracing(config: &Config) {
    if tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_str(&config.log_level).unwrap())
            .finish(),
    )
    .is_err()
    {
        // 이미 초기화된 경우 무시.
    }
}
