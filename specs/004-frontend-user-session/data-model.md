# Data Model: 프론트엔드 유저 세션 UI 통합

**Feature**: 004-frontend-user-session  
**Date**: 2025-10-18

## 개요

이 문서는 프론트엔드 WASM 애플리케이션의 클라이언트 측 데이터 모델을 정의합니다. 백엔드 API와의 계약을 준수하며, yew-router 라우팅, JWT 토큰 관리, AuthContext 기반 상태 관리를 포함합니다.

## 핵심 엔티티

### 1. Route (라우트 정의)

클라이언트 사이드 라우팅을 위한 enum. yew-router의 `#[derive(Routable)]`로 정의.

#### Route enum
```rust
use yew_router::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Home,
    
    #[at("/session/:session_id")]
    PublicSession { session_id: String },
    
    #[at("/session/:user_id/:session_id")]
    UserSession { 
        user_id: String, 
        session_id: String 
    },
    
    #[at("/my-sessions")]
    MySessions,
    
    #[not_found]
    #[at("/404")]
    NotFound,
}
```

**경로 매칭 규칙**:
- `/` → Home (세션 생성 페이지)
- `/session/abc` → PublicSession { session_id: "abc" }
- `/session/alice/abc` → UserSession { user_id: "alice", session_id: "abc" }
- `/my-sessions` → MySessions (로그인 필요)
- 기타 → NotFound (404 페이지)

**상태 전이**:
```
[브라우저 URL 변경] → [yew-router 파싱] → [Route enum variant]
                                              ↓
                                       [Switch 렌더링]
                                              ↓
                                   [해당 컴포넌트 마운트]
```

---

### 2. AuthState (로그인 상태)

사용자의 인증 상태를 나타내는 enum. AuthContext를 통해 전역 관리.

#### AuthState enum
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum AuthState {
    Anonymous,                           // 비로그인 (토큰 없음)
    Authenticated { user_id: String },   // 로그인 (토큰 있음, 만료되지 않음)
}
```

**상태 전이도**:
```
[앱 시작]
    ↓
[localStorage 조회]
    ↓
[토큰 있음?] ───No──→ Anonymous
    │
   Yes
    ↓
[JWT 디코딩]
    ↓
[만료 체크] ───Expired──→ Anonymous (토큰 삭제)
    │
  Valid
    ↓
Authenticated(user_id)

[토큰 발급 성공] → Authenticated(user_id)
[로그아웃 버튼] → Anonymous (토큰 삭제)
[401 응답] → Anonymous (토큰 삭제)
```

#### AuthContext 구조
```rust
#[derive(Clone, PartialEq)]
pub struct AuthContext {
    pub state: AuthState,
    pub login: Callback<String>,      // 토큰 저장 및 Authenticated 전환
    pub logout: Callback<()>,         // 토큰 삭제 및 Anonymous 전환
}
```

**사용 예시**:
```rust
let auth = use_context::<AuthContext>().unwrap();

match &auth.state {
    AuthState::Anonymous => {
        // 공용 세션만 생성 가능
    }
    AuthState::Authenticated { user_id } => {
        // 유저별 세션 생성/수정 가능
    }
}
```

---

### 3. Claims (JWT 토큰 Payload)

JWT 토큰에서 추출한 클레임 정보. base64 디코딩으로 읽기만 수행 (서명 검증 없음).

#### Claims 구조체
```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    pub sub: String,     // user_id (주체)
    pub exp: u64,        // 만료 시간 (Unix timestamp, 초)
    pub iat: u64,        // 발급 시간 (Unix timestamp, 초)
    pub iss: String,     // 발급자 ("realtime-svg")
}
```

**수명주기**:
```
[POST /api/auth/token] → [JWT 토큰 발급]
                              ↓
                     [localStorage 저장]
                              ↓
                      [페이지 로드 시 읽기]
                              ↓
                 [Claims::from_token() 디코딩]
                              ↓
              [만료 체크] ───Expired──→ [삭제]
                   │
                 Valid
                   ↓
           [user_id 추출] → AuthState::Authenticated
```

**메서드**:
- `Claims::from_token(token: &str) -> Result<Self, String>`: JWT 토큰에서 Claims 추출
- `is_expired(&self) -> bool`: 현재 시간 기준 만료 여부 확인
- `user_id(&self) -> &str`: sub claim 반환

---

### 4. ApiState<T> (API 요청 상태)

비동기 API 호출의 상태를 추적하는 제네릭 enum.

#### ApiState enum
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ApiState<T> {
    Idle,               // 초기 상태 (요청 전)
    Loading,            // 요청 진행 중
    Success(T),         // 성공 (응답 데이터 포함)
    Error(String),      // 실패 (에러 메시지 포함)
}
```

**상태 전이**:
```
Idle ──[API 호출]──→ Loading
                        ↓
         ┌──────────────┴──────────────┐
         │                             │
   [응답 성공]                   [응답 실패]
         ↓                             ↓
    Success(T)                    Error(String)
         │                             │
         └─────────[재시도]─────────────┘
                   ↓
                Loading
```

**사용 예시**:
```rust
let session_state = use_state(|| ApiState::Idle);

// API 호출 시작
session_state.set(ApiState::Loading);

spawn_local(async move {
    match fetch_session(session_id).await {
        Ok(data) => session_state.set(ApiState::Success(data)),
        Err(e) => session_state.set(ApiState::Error(e)),
    }
});

// UI 렌더링
match &*session_state {
    ApiState::Idle => html! { <div>{"준비됨"}</div> },
    ApiState::Loading => html! { <div>{"로딩 중..."}</div> },
    ApiState::Success(data) => html! { <div>{format!("{:?}", data)}</div> },
    ApiState::Error(msg) => html! { <div class="error">{msg}</div> },
}
```

---

### 5. SessionDetail (세션 상세 정보)

백엔드 `GET /api/session/{session_id}` 또는 `GET /api/user/{user_id}/session/{session_id}` 응답.

#### SessionDetail 구조체
```rust
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct SessionDetail {
    pub template: String,
    pub args: HashMap<String, serde_json::Value>,
}
```

**용도**: 세션 상세 페이지에서 현재 템플릿과 파라미터를 표시

**API 계약**:
```json
{
  "template": "<svg>{{text}}</svg>",
  "args": {
    "text": "Hello"
  }
}
```

---

### 6. SessionListItem (세션 목록 항목)

백엔드 `GET /api/user/{user_id}/sessions` 응답의 개별 항목.

#### SessionListItem 구조체
```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SessionListItem {
    pub session_id: String,
    pub created_at: String,      // ISO 8601 형식
    pub ttl: u64,                // 남은 TTL (초)
}
```

**API 계약**:
```json
{
  "sessions": [
    {
      "session_id": "dashboard-1",
      "created_at": "2025-10-18T10:00:00Z",
      "ttl": 3200
    }
  ]
}
```

---

### 7. TokenResponse (토큰 발급 응답)

백엔드 `POST /api/auth/token` 응답.

#### TokenResponse 구조체
```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub token: String,
}
```

**API 계약**:
```json
{
  "token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

---

### 8. SessionCreateRequest (세션 생성 요청)

`POST /api/session` 또는 `POST /api/user/{user_id}/session` 요청 본문.

#### SessionCreateRequest 구조체
```rust
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct SessionCreateRequest {
    pub session_id: String,
    pub template: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire: Option<String>,  // "1d", "3600s" 등
}
```

---

### 9. SessionUpdateRequest (세션 수정 요청)

`PUT /api/session/{session_id}` 또는 `PUT /api/user/{user_id}/session/{session_id}` 요청 본문.

#### SessionUpdateRequest 구조체
```rust
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct SessionUpdateRequest {
    pub args: HashMap<String, serde_json::Value>,
}
```

---

## 엔티티 관계도

```
┌──────────────┐
│    Route     │ (URL 경로)
└──────────────┘
       │ 렌더링
       ↓
┌──────────────────┐
│  Component Tree  │
└──────────────────┘
       │ 사용
       ↓
┌──────────────────┐        ┌─────────────┐
│   AuthContext    │───────→│  AuthState  │
│  (전역 상태)      │  포함   │ (로그인 상태) │
└──────────────────┘        └─────────────┘
       │ 제공                       │
       ↓                            ↓ 추출
┌──────────────────┐        ┌─────────────┐
│   LoginForm      │───────→│   Claims    │ (JWT payload)
│  (토큰 발급 UI)   │ 디코딩  │             │
└──────────────────┘        └─────────────┘
       │ API 호출                   │ 저장
       ↓                            ↓
┌──────────────────┐        ┌─────────────┐
│  TokenResponse   │───────→│ localStorage│
│ (백엔드 응답)     │         └─────────────┘
└──────────────────┘

┌──────────────────┐        ┌──────────────────┐
│ SessionDetail    │←──────│  SessionList     │
│ (세션 상세 정보)  │        │ (세션 목록 페이지) │
└──────────────────┘        └──────────────────┘
       ↑                            ↑
       │ GET                        │ GET
       │                            │
┌──────────────────┐        ┌──────────────────┐
│ ApiState<T>      │        │ SessionListItem  │
│ (요청 상태)       │        │ (목록 항목)       │
└──────────────────┘        └──────────────────┘
```

## 로컬 스토리지 스키마

| 키 | 값 타입 | 용도 | TTL |
|---|---------|------|-----|
| `jwt_token` | String (JWT) | 사용자 인증 토큰 | 백엔드 exp claim (24시간) |

**접근 패턴**:
- 앱 시작: localStorage 읽기 → Claims 추출 → AuthState 복원
- 로그인: TokenResponse.token → localStorage 저장
- 로그아웃: localStorage 삭제
- 401 응답: localStorage 삭제 (자동 로그아웃)

## 컴포넌트 상태 패턴

### 로컬 상태 (use_state)
- 폼 입력값 (user_id, session_id, template, args)
- API 요청 상태 (ApiState<T>)
- UI 표시 상태 (에러 메시지, 로딩 스피너)

### 전역 상태 (use_context)
- AuthState (모든 컴포넌트에서 접근)
- login/logout Callback (헤더, 폼 등에서 사용)

## 타입 안전성 보장

1. **Route enum**: 잘못된 경로를 컴파일 타임에 감지
2. **AuthState enum**: 로그인/비로그인 상태를 명시적으로 구분
3. **ApiState<T>**: 요청 상태와 성공 데이터를 타입으로 보장
4. **Deserialize**: JSON 응답을 강타입 구조체로 자동 변환
5. **Serialize**: 요청 본문을 컴파일 타임에 검증

## 에러 처리 전략

- **네트워크 오류**: `ApiState::Error("Network error: ...")`
- **JSON 파싱 오류**: `ApiState::Error("Invalid response format")`
- **401 Unauthorized**: AuthState → Anonymous, localStorage 삭제
- **403 Forbidden**: `ApiState::Error("권한이 없습니다")`
- **404 Not Found**: `ApiState::Error("세션을 찾을 수 없습니다")`
- **localStorage 없음**: 공용 세션 폴백, UI 경고 표시
