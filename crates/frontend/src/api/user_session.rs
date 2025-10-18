use crate::types::{UserSessionCreateRequest, SessionDetail, SessionUpdateRequest, SessionListResponse};
use crate::api::AuthenticatedRequest;

#[derive(serde::Deserialize)]
pub struct CreateSessionResponse {
    pub user_id: String,
    pub session_id: String,
}

pub async fn create_user_session(
    user_id: &str,
    request: UserSessionCreateRequest,
) -> Result<CreateSessionResponse, String> {
    let auth_req = AuthenticatedRequest::new();
    
    let response = auth_req
        .post(&format!("/api/user/{}/session", user_id))
        .header("Content-Type", "application/json")
        .json(&request)
        .map_err(|e| format!("요청 생성 실패: {}", e))?
        .send()
        .await
        .map_err(|e| format!("네트워크 오류: {}", e))?;
    
    match response.status() {
        201 => {
            let create_response: CreateSessionResponse = response
                .json()
                .await
                .map_err(|e| format!("응답 파싱 실패: {}", e))?;
            Ok(create_response)
        }
        401 => Err("로그인이 필요합니다".to_string()),
        403 => Err("권한이 없습니다".to_string()),
        409 => Err("이미 존재하는 세션 ID입니다".to_string()),
        status => Err(format!("세션 생성 실패 ({})", status)),
    }
}

pub async fn get_user_session_detail(
    user_id: &str,
    session_id: &str,
) -> Result<SessionDetail, String> {
    let auth_req = AuthenticatedRequest::new();
    
    let response = auth_req
        .get(&format!("/api/user/{}/session/{}", user_id, session_id))
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
        401 => Err("로그인이 필요합니다".to_string()),
        403 => Err("권한이 없습니다".to_string()),
        404 => Err("세션을 찾을 수 없습니다".to_string()),
        status => Err(format!("세션 조회 실패 ({})", status)),
    }
}

pub async fn update_user_session(
    user_id: &str,
    session_id: &str,
    request: SessionUpdateRequest,
) -> Result<(), String> {
    let auth_req = AuthenticatedRequest::new();
    
    let response = auth_req
        .put(&format!("/api/user/{}/session/{}", user_id, session_id))
        .header("Content-Type", "application/json")
        .json(&request)
        .map_err(|e| format!("요청 생성 실패: {}", e))?
        .send()
        .await
        .map_err(|e| format!("네트워크 오류: {}", e))?;
    
    match response.status() {
        204 => Ok(()),
        401 => Err("로그인이 필요합니다".to_string()),
        403 => Err("권한이 없습니다".to_string()),
        404 => Err("세션을 찾을 수 없습니다".to_string()),
        status => Err(format!("세션 수정 실패 ({})", status)),
    }
}

pub async fn list_user_sessions(user_id: &str) -> Result<SessionListResponse, String> {
    let auth_req = AuthenticatedRequest::new();
    
    let response = auth_req
        .get(&format!("/api/user/{}/session", user_id))
        .send()
        .await
        .map_err(|e| format!("네트워크 오류: {}", e))?;
    
    match response.status() {
        200 => {
            let list_response: SessionListResponse = response
                .json()
                .await
                .map_err(|e| format!("응답 파싱 실패: {}", e))?;
            Ok(list_response)
        }
        401 => Err("로그인이 필요합니다".to_string()),
        403 => Err("권한이 없습니다".to_string()),
        status => Err(format!("세션 목록 조회 실패 ({})", status)),
    }
}
