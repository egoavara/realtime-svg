use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub mod auth;
pub mod browser_engine;
pub mod config;
pub mod errors;
pub mod jwt;
pub mod session_data;
pub mod share;
pub mod state;
pub mod user_data;
pub mod whoami;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse<T: Serialize> {
    pub items: Vec<T>,
}

/// 세션 관련 응답.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionInfo {
    pub session_id: String,
}

/// 세션 관련 응답.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionDetailInfo {
    pub session_id: String,
    pub template: String,
    pub args: HashMap<String, serde_json::Value>,
}

/// 브로드캐스트되는 SVG 프레임.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SvgFrame {
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

impl SvgFrame {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            timestamp: Utc::now(),
        }
    }
}
