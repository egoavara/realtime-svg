pub mod auth;
pub mod public_session;
pub mod user_session;

use crate::auth::storage::{LocalTokenStorage, TokenStorage};
use gloo_net::http::{RequestBuilder, Response};

/// Authorization 헤더를 자동으로 추가하는 Request 빌더
pub struct AuthenticatedRequest {
    storage: LocalTokenStorage,
}

impl AuthenticatedRequest {
    pub fn new() -> Self {
        Self {
            storage: LocalTokenStorage::new(),
        }
    }

    /// GET 요청 (인증 필요)
    pub fn get(&self, url: &str) -> RequestBuilder {
        let mut req = RequestBuilder::new(url).method(gloo_net::http::Method::GET);

        if let Some(token) = self.storage.get_token() {
            req = req.header("Authorization", &format!("Bearer {}", token));
        }

        req
    }

    /// POST 요청 (인증 필요)
    pub fn post(&self, url: &str) -> RequestBuilder {
        let mut req = RequestBuilder::new(url).method(gloo_net::http::Method::POST);

        if let Some(token) = self.storage.get_token() {
            req = req.header("Authorization", &format!("Bearer {}", token));
        }

        req
    }

    /// PUT 요청 (인증 필요)
    pub fn put(&self, url: &str) -> RequestBuilder {
        let mut req = RequestBuilder::new(url).method(gloo_net::http::Method::PUT);

        if let Some(token) = self.storage.get_token() {
            req = req.header("Authorization", &format!("Bearer {}", token));
        }

        req
    }
}

/// 에러 응답 본문 파싱
#[derive(serde::Deserialize)]
struct ErrorResponse {
    error: String,
}

/// API 응답 처리 (401/403 자동 감지)
pub async fn handle_response(response: Response) -> Result<Response, String> {
    match response.status() {
        401 => {
            log::warn!("401 Unauthorized - token expired or invalid");
            // 토큰 삭제
            let storage = LocalTokenStorage::new();
            let _ = storage.remove_token();
            Err("토큰이 만료되었습니다. 다시 로그인하세요".to_string())
        }
        403 => {
            log::warn!("403 Forbidden - insufficient permissions");
            Err("권한이 없습니다. 세션 소유자만 수정할 수 있습니다".to_string())
        }
        404 => Err("세션을 찾을 수 없습니다".to_string()),
        409 => Err("이미 존재하는 세션 ID입니다".to_string()),
        status if status >= 400 => {
            // JSON 에러 본문 파싱 시도
            if let Ok(error_body) = response.json::<ErrorResponse>().await {
                Err(format!("[{}] {}", status, error_body.error))
            } else {
                Err(format!("요청 실패 ({})", status))
            }
        }
        _ => Ok(response),
    }
}
