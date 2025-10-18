use anyhow::{Context, Result};
use clap::Parser;
use figment::{
    providers::{Env, Format, Serialized, Yaml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{f64::consts::E, fmt};
use tracing::info;

/// 서버 실행에 필요한 모든 설정값을 담는 구조체
///
/// 설정 우선순위: CLI 옵션 > 환경 변수 > 설정 파일 > 기본값
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    /// Redis 서버 연결 URL (예: redis://127.0.0.1:6379/)
    pub redis_url: String,
    /// HTTP 서버 바인딩 주소 (예: 127.0.0.1, 0.0.0.0)
    pub host: String,
    /// HTTP 서버 포트 번호 (1-65535)
    pub port: u16,
    /// 로깅 레벨 (info, debug, warn, error)
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            redis_url: "redis://127.0.0.1/".to_string(),
            host: "127.0.0.1".to_string(),
            port: 3000,
            log_level: "info".to_string(),
        }
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Config")
            .field("redis_url", &"***REDACTED***")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("log_level", &self.log_level)
            .finish()
    }
}

/// 커맨드라인 인자 파싱 구조체
///
/// Clap derive API를 사용하여 CLI 옵션과 환경 변수를 자동으로 파싱합니다.
/// 모든 필드는 Option<T>로 선택적이며, 제공되지 않으면 다음 레이어(ENV, Config)로 fallback됩니다.
#[derive(Parser, Serialize)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Redis 서버 URL (환경 변수: REDIS_URL)
    #[arg(long, env = "REDIS_URL", help = "Redis server URL")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_url: Option<String>,

    /// 서버 바인딩 주소 (환경 변수: HOST)
    #[arg(long, env = "HOST", help = "Server bind address")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,

    /// 서버 포트 (환경 변수: PORT)
    #[arg(long, env = "PORT", help = "Server port")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    /// 로깅 레벨 (환경 변수: LOG_LEVEL)
    #[arg(
        long,
        env = "LOG_LEVEL",
        help = "Logging level (info, debug, warn, error)"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<String>,

    /// 설정 파일 경로 (기본값: config.yaml)
    #[arg(long, help = "Path to configuration file, default: config.yaml")]
    #[serde(skip)]
    pub config: Option<PathBuf>,
}

fn resolve_config_path(config_arg: &Option<PathBuf>) -> Result<PathBuf> {
    match config_arg {
        None => Ok(PathBuf::from("config.yaml")),
        Some(config_arg) => {
            if config_arg.is_absolute() {
                return Ok(config_arg.to_path_buf());
            }

            let exe_dir = std::env::current_exe()?
                .parent()
                .context("실행 파일 경로를 찾을 수 없습니다")?
                .to_path_buf();
            let filepath = exe_dir.join(config_arg);
            if filepath.exists() {
                return Ok(filepath);
            }
            Err(anyhow::anyhow!(
                "설정 파일을 찾을 수 없습니다: {:?}",
                filepath
            ))
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let cli_args = CliArgs::parse();
        let config_path = resolve_config_path(&cli_args.config)?;
        let config: Config = Figment::new()
            .merge(Serialized::defaults(Config::default()))
            .merge(Yaml::file(config_path))
            .merge(Env::raw())
            .merge(Serialized::defaults(cli_args))
            .extract()
            .context("설정 파일 로딩 실패")?;

        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.port == 0 {
            anyhow::bail!("포트는 1-65535 범위여야 합니다: {}", self.port);
        }

        if !self.redis_url.starts_with("redis://") && !self.redis_url.starts_with("rediss://") {
            anyhow::bail!(
                "Redis URL은 redis:// 또는 rediss:// 로 시작해야 합니다: {}",
                self.redis_url
            );
        }

        if self.host.is_empty() {
            anyhow::bail!("호스트는 빈 문자열일 수 없습니다");
        }

        Ok(())
    }
}
