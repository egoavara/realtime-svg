use gloo_net::http::Request;
use crate::types::TokenResponse;

#[derive(serde::Serialize)]
struct TokenRequest {
    user_id: String,
    password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ttl_seconds: Option<u64>,
}

pub async fn request_token(user_id: String, password: String, ttl_seconds: Option<u64>) -> Result<String, String> {
    let request_body = TokenRequest { user_id, password, ttl_seconds };
    
    let response = Request::post("/api/auth/token")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .map_err(|e| format!("요청 생성 실패: {}", e))?
        .send()
        .await
        .map_err(|e| format!("네트워크 오류: {}", e))?;
    
    if response.ok() {
        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| format!("응답 파싱 실패: {}", e))?;
        
        Ok(token_response.token)
    } else {
        let status = response.status();
        Err(format!("토큰 발급 실패 ({})", status))
    }
}
