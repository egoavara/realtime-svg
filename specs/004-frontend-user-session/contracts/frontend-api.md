# Frontend API Contracts

**Feature**: 004-frontend-user-session  
**Date**: 2025-10-18  
**Purpose**: 프론트엔드 WASM 앱이 호출하는 백엔드 API 엔드포인트 계약 정의

## 개요

이 문서는 프론트엔드가 백엔드와 통신하는 HTTP API 계약을 정의합니다. 모든 엔드포인트는 spec 002의 `contracts/auth-api.yaml` 및 `contracts/user-session-api.yaml`을 준수합니다.

## API 엔드포인트 목록

### 1. 토큰 발급 API

#### `POST /api/auth/token`

**목적**: 사용자 ID를 기반으로 JWT 토큰 발급

**인증**: 불필요 ❌

**요청**:
```http
POST /api/auth/token HTTP/1.1
Content-Type: application/json

{
  "user_id": "alice"
}
```

**요청 스키마**:
```rust
struct TokenRequest {
    user_id: String,
}
```

**성공 응답** (200 OK):
```json
{
  "token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhbGljZSIsImV4cCI6MTczNDQ4MDAwMCwiaWF0IjoxNzM0NDAwMDAwLCJpc3MiOiJyZWFsdGltZS1zdmcifQ.signature..."
}
```

**에러 응답**:
- `400 Bad Request`: user_id 누락 또는 빈 문자열
  ```json
  {"error": "Invalid user_id"}
  ```

**프론트엔드 처리**:
```rust
use crate::api::auth::request_token;

let token = request_token("alice").await?;
// localStorage에 저장
storage.set_token(&token)?;
// AuthContext 업데이트
auth.login.emit(token);
```

---

### 2. 유저별 세션 생성 API

#### `POST /api/user/{user_id}/session`

**목적**: 특정 사용자의 세션 생성

**인증**: 필수 ✅ (Authorization: Bearer {token})

**요청**:
```http
POST /api/user/alice/session HTTP/1.1
Authorization: Bearer eyJhbGci...
Content-Type: application/json

{
  "session_id": "dashboard-1",
  "template": "<svg>{{text}}</svg>",
  "args": {
    "text": "Hello"
  },
  "expire_seconds": 3600
}
```

**요청 스키마**:
```rust
struct UserSessionCreateRequest {
    session_id: String,
    template: String,
    args: Option<HashMap<String, serde_json::Value>>,
    expire_seconds: Option<u64>,
}
```

**성공 응답** (201 Created):
```json
{
  "user_id": "alice",
  "session_id": "dashboard-1"
}
```

**에러 응답**:
- `401 Unauthorized`: 토큰 없음, 만료, 또는 서명 불일치
  ```json
  {"error": "Missing Authorization header"}
  ```
- `403 Forbidden`: 토큰의 user_id와 URL의 user_id 불일치
  ```json
  {"error": "User bob cannot modify sessions of user alice"}
  ```
- `409 Conflict`: 동일한 user_id + session_id 조합이 이미 존재
  ```json
  {"error": "Session already exists"}
  ```

**프론트엔드 처리**:
```rust
use crate::api::user_session::create_session;

match auth.state {
    AuthState::Authenticated { user_id } => {
        create_session(&user_id, "dashboard-1", template, args).await?;
        navigator.push(&Route::UserSession { 
            user_id: user_id.clone(), 
            session_id: "dashboard-1".to_string() 
        });
    }
    AuthState::Anonymous => {
        // "로그인이 필요합니다" 메시지 표시
    }
}
```

---

### 3. 유저별 세션 수정 API

#### `PUT /api/user/{user_id}/session/{session_id}`

**목적**: 세션 파라미터 업데이트 (템플릿 변경 불가)

**인증**: 필수 ✅

**요청**:
```http
PUT /api/user/alice/session/dashboard-1 HTTP/1.1
Authorization: Bearer eyJhbGci...
Content-Type: application/json

{
  "args": {
    "text": "Updated"
  }
}
```

**요청 스키마**:
```rust
struct SessionUpdateRequest {
    args: HashMap<String, serde_json::Value>,
}
```

**성공 응답** (204 No Content):
```
(본문 없음)
```

**에러 응답**:
- `401 Unauthorized`: 토큰 만료
- `403 Forbidden`: 세션 소유자가 아님
- `404 Not Found`: 세션이 존재하지 않음
  ```json
  {"error": "Session not found"}
  ```

**프론트엔드 처리**:
```rust
use crate::api::user_session::update_session;

// 401 응답 시 자동 로그아웃
match update_session(&user_id, &session_id, args).await {
    Ok(_) => {
        // 성공 메시지 표시
        // 스트림 미리보기 새로고침
    }
    Err(e) if e.contains("Unauthorized") => {
        auth.logout.emit(());  // 자동 로그아웃
    }
    Err(e) => {
        // 에러 메시지 표시
    }
}
```

---

### 4. 유저별 세션 조회 API

#### `GET /api/user/{user_id}/session/{session_id}`

**목적**: 세션의 현재 템플릿과 파라미터 조회

**인증**: 불필요 ❌ (읽기는 공개)

**요청**:
```http
GET /api/user/alice/session/dashboard-1 HTTP/1.1
```

**성공 응답** (200 OK):
```json
{
  "template": "<svg>{{text}}</svg>",
  "args": {
    "text": "Hello"
  }
}
```

**에러 응답**:
- `404 Not Found`: 세션이 존재하지 않음 또는 TTL 만료

**프론트엔드 처리**:
```rust
use crate::api::user_session::get_session_detail;

let detail = get_session_detail(&user_id, &session_id).await?;
// 템플릿을 읽기 전용 textarea에 표시
// args를 편집 가능한 JSON textarea에 표시
```

---

### 5. 유저별 세션 목록 조회 API

#### `GET /api/user/{user_id}/sessions`

**목적**: 특정 사용자가 생성한 모든 세션 목록 조회

**인증**: 필수 ✅

**요청**:
```http
GET /api/user/alice/sessions HTTP/1.1
Authorization: Bearer eyJhbGci...
```

**성공 응답** (200 OK):
```json
{
  "sessions": [
    {
      "session_id": "dashboard-1",
      "created_at": "2025-10-18T10:00:00Z",
      "ttl": 3200
    },
    {
      "session_id": "widget-status",
      "created_at": "2025-10-18T09:30:00Z",
      "ttl": 3600
    }
  ]
}
```

**응답 스키마**:
```rust
struct SessionListResponse {
    sessions: Vec<SessionListItem>,
}

struct SessionListItem {
    session_id: String,
    created_at: String,  // ISO 8601
    ttl: u64,            // 초
}
```

**에러 응답**:
- `401 Unauthorized`: 토큰 만료
- `403 Forbidden`: 다른 사용자의 세션 목록 조회 시도

**프론트엔드 처리**:
```rust
use crate::api::user_session::list_sessions;

let sessions = list_sessions(&user_id).await?;
// 테이블 또는 카드 형태로 렌더링
// 각 항목 클릭 시 UserSession 라우트로 이동
```

---

### 6. 공용 세션 생성 API (하위 호환성)

#### `POST /api/session`

**목적**: 소유자가 없는 공용 세션 생성 (기존 방식)

**인증**: 불필요 ❌

**요청**:
```http
POST /api/session HTTP/1.1
Content-Type: application/json

{
  "session_id": "public-demo",
  "template": "<svg>{{color}}</svg>",
  "args": {
    "color": "red"
  },
  "expire": "1d"
}
```

**요청 스키마**:
```rust
struct PublicSessionCreateRequest {
    session_id: String,
    template: String,
    args: Option<HashMap<String, serde_json::Value>>,
    expire: Option<String>,  // "1d", "3600s" 등
}
```

**성공 응답** (201 Created):
```json
{
  "session_id": "public-demo"
}
```

**에러 응답**:
- `409 Conflict`: 동일한 session_id가 이미 존재

**프론트엔드 처리**:
```rust
use crate::api::public_session::create_session;

// 로그인 여부와 관계없이 호출 가능
create_session("public-demo", template, args).await?;
navigator.push(&Route::PublicSession { 
    session_id: "public-demo".to_string() 
});
```

---

### 7. 공용 세션 수정 API

#### `PUT /api/session/{session_id}`

**목적**: 공용 세션 파라미터 업데이트

**인증**: 불필요 ❌

**요청**:
```http
PUT /api/session/public-demo HTTP/1.1
Content-Type: application/json

{
  "args": {
    "color": "blue"
  }
}
```

**성공 응답** (204 No Content)

**에러 응답**:
- `404 Not Found`: 세션이 존재하지 않음

---

### 8. 공용 세션 조회 API

#### `GET /api/session/{session_id}`

**목적**: 공용 세션 상세 정보 조회

**인증**: 불필요 ❌

**요청**:
```http
GET /api/session/public-demo HTTP/1.1
```

**성공 응답** (200 OK):
```json
{
  "template": "<svg>{{color}}</svg>",
  "args": {
    "color": "red"
  }
}
```

---

## 공통 에러 응답 형식

모든 에러 응답은 다음 JSON 형식을 따릅니다:

```json
{
  "error": "Error message"
}
```

**HTTP 상태 코드 매핑**:
- `400`: 잘못된 요청 본문 (필수 필드 누락, 타입 오류)
- `401`: 인증 실패 (토큰 없음, 만료, 서명 오류)
- `403`: 권한 없음 (다른 사용자의 세션 수정 시도)
- `404`: 리소스 없음 (세션 미존재)
- `409`: 충돌 (세션 ID 중복)
- `500`: 서버 내부 오류

## 프론트엔드 에러 처리 패턴

```rust
use crate::api::handle_response;

async fn call_api() -> Result<ResponseData, String> {
    let response = Request::get(url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    let response = handle_response(response).await?;  // 401/403 자동 처리
    
    response.json::<ResponseData>()
        .await
        .map_err(|e| format!("Invalid response: {}", e))
}
```

**handle_response 로직**:
- `401` → localStorage 토큰 삭제, AuthState → Anonymous, "토큰이 만료되었습니다" 에러 반환
- `403` → "권한이 없습니다" 에러 반환
- `404` → "세션을 찾을 수 없습니다" 에러 반환
- `409` → "이미 존재하는 세션 ID입니다" 에러 반환
- 기타 4xx/5xx → JSON 본문에서 error 필드 추출

## CORS 설정 요구사항

백엔드는 다음 CORS 헤더를 응답해야 합니다:

```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, PUT, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization
```

trunk serve (개발 서버)는 기본적으로 `http://127.0.0.1:8080`에서 실행되므로, 백엔드가 `localhost:3000`에서 실행 중이라면 CORS 설정이 필요합니다.

## 요약

| 엔드포인트 | 메서드 | 인증 | 용도 |
|-----------|--------|------|------|
| `/api/auth/token` | POST | ❌ | JWT 토큰 발급 |
| `/api/user/{user_id}/session` | POST | ✅ | 유저별 세션 생성 |
| `/api/user/{user_id}/session/{session_id}` | PUT | ✅ | 유저별 세션 수정 |
| `/api/user/{user_id}/session/{session_id}` | GET | ❌ | 유저별 세션 조회 |
| `/api/user/{user_id}/sessions` | GET | ✅ | 유저별 세션 목록 |
| `/api/session` | POST | ❌ | 공용 세션 생성 |
| `/api/session/{session_id}` | PUT | ❌ | 공용 세션 수정 |
| `/api/session/{session_id}` | GET | ❌ | 공용 세션 조회 |

**다음 단계**: quickstart.md 작성, agent context 업데이트
