use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// API request state tracking
#[derive(Debug, Clone, PartialEq)]
pub enum ApiState<T> {
    Idle,
    Loading,
    Success(T),
    Error(String),
}

impl<T> Default for ApiState<T> {
    fn default() -> Self {
        Self::Idle
    }
}

/// JWT token response from /api/auth/token
#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub token: String,
}

/// Session detail response
#[derive(Debug, Clone, Deserialize)]
pub struct SessionDetail {
    pub template: String,
    pub args: HashMap<String, serde_json::Value>,
}

/// User session create request
#[derive(Debug, Clone, Serialize)]
pub struct UserSessionCreateRequest {
    pub session_id: String,
    pub template: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_seconds: Option<u64>,
}

/// Session update request
#[derive(Debug, Clone, Serialize)]
pub struct SessionUpdateRequest {
    pub args: HashMap<String, serde_json::Value>,
}

/// Session list item
#[derive(Debug, Clone, Deserialize)]
pub struct SessionListItem {
    pub session_id: String,
}

/// Session list response
#[derive(Debug, Clone, Deserialize)]
pub struct SessionListResponse {
    pub items: Vec<SessionListItem>,
}

/// Public session create request
#[derive(Debug, Clone, Serialize)]
pub struct PublicSessionCreateRequest {
    pub session_id: String,
    pub template: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire: Option<String>,
}
