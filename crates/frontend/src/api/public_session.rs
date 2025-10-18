use gloo_net::http::Request;
use crate::types::{PublicSessionCreateRequest, SessionDetail, SessionUpdateRequest};

#[derive(serde::Deserialize)]
pub struct CreatePublicSessionResponse {
    pub session_id: String,
}

pub async fn create_public_session(
    request: PublicSessionCreateRequest,
) -> Result<String, String> {
    let response = Request::post("/api/session")
        .header("Content-Type", "application/json")
        .json(&request)
        .map_err(|e| format!("요청 생성 실패: {}", e))?
        .send()
        .await
        .map_err(|e| format!("네트워크 오류: {}", e))?;
    
    match response.status() {
        201 => {
            let create_response: CreatePublicSessionResponse = response
                .json()
                .await
                .map_err(|e| format!("응답 파싱 실패: {}", e))?;
            Ok(create_response.session_id)
        }
        409 => Err("이미 존재하는 세션 ID입니다".to_string()),
        status => Err(format!("세션 생성 실패 ({})", status)),
    }
}

pub async fn get_public_session_detail(session_id: &str) -> Result<SessionDetail, String> {
    let response = Request::get(&format!("/api/session/{}", session_id))
        .send()
        .await
        .map_err(|e| format!("네트워크 오류: {}", e))?;
    
    match response.status() {
        200 => {
            let detail: SessionDetail = response
                .json()
                .await
                .map_err(|e| format!("응답 파싱 실패: {}", e))?;
            Ok(detail)
        }
        404 => Err("세션을 찾을 수 없습니다".to_string()),
        status => Err(format!("세션 조회 실패 ({})", status)),
    }
}

pub async fn update_public_session(
    session_id: &str,
    request: SessionUpdateRequest,
) -> Result<(), String> {
    let response = Request::put(&format!("/api/session/{}", session_id))
        .header("Content-Type", "application/json")
        .json(&request)
        .map_err(|e| format!("요청 생성 실패: {}", e))?
        .send()
        .await
        .map_err(|e| format!("네트워크 오류: {}", e))?;
    
    match response.status() {
        200 => Ok(()),
        404 => Err("세션을 찾을 수 없습니다".to_string()),
        status => Err(format!("세션 수정 실패 ({})", status)),
    }
}
