# Research: 프론트엔드 유저 세션 UI 통합 기술 조사

**Date**: 2025-10-18  
**Feature**: 004-frontend-user-session  
**Purpose**: yew-router 기반 SPA 라우팅, JWT 토큰 관리, localStorage 활용 방법 연구

## 목차

1. [yew-router 0.18 통합](#1-yew-router-018-통합)
2. [JWT 디코딩 (클라이언트 측)](#2-jwt-디코딩-클라이언트-측)
3. [localStorage 안전한 래핑](#3-localstorage-안전한-래핑)
4. [gloo-net 인증 헤더 관리](#4-gloo-net-인증-헤더-관리)
5. [Yew 컴포넌트 상태 관리](#5-yew-컴포넌트-상태-관리)

---

## 1. yew-router 0.18 통합

### Decision
**yew-router 0.18을 사용하여 클라이언트 사이드 라우팅 구현, `#[derive(Routable)]`로 Route enum 정의**

### Rationale
- **타입 안전성**: enum 기반 라우팅으로 컴파일 타임에 잘못된 경로 감지
- **동적 파라미터**: `/session/:user_id/:session_id` 형태의 경로 파라미터를 구조체 필드로 자동 파싱
- **History API 통합**: 브라우저 히스토리와 자동 동기화, 뒤로가기/앞으로가기 지원
- **Link 컴포넌트**: `<Link<Route> to={Route::Home}>`로 타입 안전한 네비게이션
- **404 처리**: `#[not_found]` 속성으로 잘못된 경로 자동 처리

### Alternatives Considered
- **수동 URL 파싱**: `window.location.pathname()` 직접 파싱 → 타입 안전성 없음, 보일러플레이트 코드 증가
- **gloo-history 직접 사용**: 저수준 API → yew-router가 더 높은 추상화 제공
- **해시 라우팅 (#/)**: BrowserRouter보다 SEO 불리, URL이 덜 직관적

### Implementation Details

**의존성 추가 (Cargo.toml)**:
```toml
[dependencies]
yew = { version = "0.21", features = ["csr"] }
yew-router = "0.18"
```

**Route enum 정의 (src/routes.rs)**:
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

**BrowserRouter 설정 (src/lib.rs)**:
```rust
use yew::prelude::*;
use yew_router::prelude::*;
use crate::routes::Route;

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <HomePage /> },
        Route::PublicSession { session_id } => {
            html! { <SessionDetailPage {session_id} is_user_session={false} /> }
        }
        Route::UserSession { user_id, session_id } => {
            html! { <SessionDetailPage user_id={Some(user_id)} {session_id} is_user_session={true} /> }
        }
        Route::MySessions => html! { <MySessionsPage /> },
        Route::NotFound => html! { <NotFoundPage /> },
    }
}
```

**Link 컴포넌트 사용**:
```rust
use yew_router::prelude::*;

html! {
    <Link<Route> to={Route::Home}>{"홈으로"}</Link<Route>>
    <Link<Route> to={Route::MySessions}>{"내 세션 목록"}</Link<Route>>
}
```

**프로그래매틱 네비게이션 (use_navigator hook)**:
```rust
use yew_router::prelude::*;

#[function_component(CreateSessionForm)]
fn create_session_form() -> Html {
    let navigator = use_navigator().unwrap();
    
    let on_success = {
        let navigator = navigator.clone();
        Callback::from(move |session_id: String| {
            navigator.push(&Route::PublicSession { session_id });
        })
    };
    
    // ...
}
```

**URL 파라미터 구분 로직**:
- `/session/{session_id}` → 세그먼트 1개 → PublicSession
- `/session/{user_id}/{session_id}` → 세그먼트 2개 → UserSession
- yew-router가 자동으로 매칭하여 적절한 enum variant로 파싱

---

## 2. JWT 디코딩 (클라이언트 측)

### Decision
**base64 크레이트로 JWT payload를 디코딩하여 Claims 추출, 서명 검증은 백엔드에서만 수행**

### Rationale
- **경량화**: 전체 JWT 검증 라이브러리(jsonwebtoken) 불필요, WASM 번들 크기 절감
- **충분한 보안**: 서명 검증은 백엔드에서 수행하므로 클라이언트는 payload 읽기만 필요
- **만료 시간 체크**: exp claim을 읽어 클라이언트 측에서 만료 여부 미리 판단 가능
- **user_id 추출**: sub claim에서 user_id를 읽어 UI에 표시 및 API 경로 생성

### Alternatives Considered
- **jsonwebtoken 크레이트 사용**: 서명 검증까지 수행 → WASM에서 불필요한 무거운 의존성
- **텍스트 분할만 사용**: base64 디코딩 없이 '.' 분할 → JSON 파싱 불가능
- **백엔드에 user_id 요청**: 별도 API 호출 필요 → 불필요한 네트워크 오버헤드

### Implementation Details

**의존성 추가 (Cargo.toml)**:
```toml
[dependencies]
base64 = "0.22"
serde = { workspace = true }
serde_json = { workspace = true }
```

**Claims 구조체 정의 (src/auth/token.rs)**:
```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    pub sub: String,     // user_id
    pub exp: u64,        // 만료 시간 (Unix timestamp)
    pub iat: u64,        // 발급 시간
    pub iss: String,     // 발급자
}

impl Claims {
    /// JWT 토큰에서 Claims 추출
    pub fn from_token(token: &str) -> Result<Self, String> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid JWT format: expected 3 parts".to_string());
        }
        
        // payload는 두 번째 부분 (header.payload.signature)
        let payload_b64 = parts[1];
        
        // base64 디코딩 (URL-safe, padding 없음)
        use base64::{Engine as _, engine::general_purpose};
        let payload_bytes = general_purpose::URL_SAFE_NO_PAD
            .decode(payload_b64)
            .map_err(|e| format!("Base64 decode failed: {}", e))?;
        
        // JSON 파싱
        let claims: Claims = serde_json::from_slice(&payload_bytes)
            .map_err(|e| format!("JSON parse failed: {}", e))?;
        
        Ok(claims)
    }
    
    /// 토큰 만료 여부 확인
    pub fn is_expired(&self) -> bool {
        use js_sys::Date;
        let now = (Date::now() / 1000.0) as u64;  // 밀리초 → 초
        now >= self.exp
    }
    
    /// user_id 추출
    pub fn user_id(&self) -> &str {
        &self.sub
    }
}
```

**사용 예시**:
```rust
let token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhbGljZSIsImV4cCI6MTczNDQ4MDAwMCwiaWF0IjoxNzM0NDAwMDAwLCJpc3MiOiJyZWFsdGltZS1zdmcifQ.signature...";

match Claims::from_token(token) {
    Ok(claims) => {
        if claims.is_expired() {
            log::warn!("Token expired for user: {}", claims.user_id());
            // localStorage에서 삭제
        } else {
            log::info!("Valid token for user: {}", claims.user_id());
        }
    }
    Err(e) => log::error!("Failed to parse token: {}", e),
}
```

**주의사항**:
- 클라이언트에서는 **서명 검증을 하지 않으므로** 토큰의 무결성을 신뢰해서는 안 됨
- 모든 민감한 작업(세션 생성/수정)은 백엔드에서 토큰 검증 후 수행
- 클라이언트 측 만료 시간 체크는 **UX 개선용**이며, 백엔드가 최종 검증 권한 보유

---

## 3. localStorage 안전한 래핑

### Decision
**web-sys Storage API를 래핑한 타입 안전한 TokenStorage 모듈 구현**

### Rationale
- **에러 핸들링**: localStorage 비활성화, QuotaExceededError 등을 Result 타입으로 안전하게 처리
- **타입 안전성**: String 대신 Option<String> 반환으로 None 케이스 명시
- **테스트 가능성**: MockStorage trait으로 단위 테스트 시 localStorage 모킹 가능
- **단일 책임**: 토큰 저장/읽기/삭제 로직을 한 곳에 캡슐화

### Alternatives Considered
- **직접 window.localStorage 호출**: 에러 처리가 모든 곳에 흩어짐, 테스트 불가능
- **gloo-storage 크레이트**: 제네릭 저장소 추상화 → JWT 토큰에는 과도한 추상화
- **쿠키 사용**: CORS 문제, HttpOnly 속성 사용 불가 (JS 접근 필요)

### Implementation Details

**의존성 추가 (Cargo.toml)**:
```toml
[dependencies]
web-sys = { version = "0.3", features = ["Window", "Storage"] }
```

**TokenStorage trait 및 구현 (src/auth/storage.rs)**:
```rust
use web_sys::{window, Storage};

const TOKEN_KEY: &str = "jwt_token";

/// 토큰 저장소 추상화 (테스트용 모킹 가능)
pub trait TokenStorage {
    fn get_token(&self) -> Option<String>;
    fn set_token(&self, token: &str) -> Result<(), String>;
    fn remove_token(&self) -> Result<(), String>;
}

/// localStorage 기반 구현
pub struct LocalTokenStorage;

impl LocalTokenStorage {
    pub fn new() -> Self {
        Self
    }
    
    /// localStorage 인스턴스 가져오기 (None이면 비활성화됨)
    fn get_storage(&self) -> Option<Storage> {
        window()?.local_storage().ok()?
    }
}

impl TokenStorage for LocalTokenStorage {
    fn get_token(&self) -> Option<String> {
        let storage = self.get_storage()?;
        storage.get_item(TOKEN_KEY).ok()?
    }
    
    fn set_token(&self, token: &str) -> Result<(), String> {
        let storage = self.get_storage()
            .ok_or("localStorage not available")?;
        
        storage.set_item(TOKEN_KEY, token)
            .map_err(|e| {
                let error_msg = format!("{:?}", e);
                if error_msg.contains("QuotaExceededError") {
                    "Storage quota exceeded".to_string()
                } else {
                    format!("Failed to set token: {}", error_msg)
                }
            })
    }
    
    fn remove_token(&self) -> Result<(), String> {
        let storage = self.get_storage()
            .ok_or("localStorage not available")?;
        
        storage.remove_item(TOKEN_KEY)
            .map_err(|e| format!("Failed to remove token: {:?}", e))
    }
}

/// 테스트용 Mock 구현
#[cfg(test)]
pub struct MockTokenStorage {
    token: std::cell::RefCell<Option<String>>,
}

#[cfg(test)]
impl MockTokenStorage {
    pub fn new() -> Self {
        Self {
            token: std::cell::RefCell::new(None),
        }
    }
}

#[cfg(test)]
impl TokenStorage for MockTokenStorage {
    fn get_token(&self) -> Option<String> {
        self.token.borrow().clone()
    }
    
    fn set_token(&self, token: &str) -> Result<(), String> {
        *self.token.borrow_mut() = Some(token.to_string());
        Ok(())
    }
    
    fn remove_token(&self) -> Result<(), String> {
        *self.token.borrow_mut() = None;
        Ok(())
    }
}
```

**사용 예시**:
```rust
use crate::auth::storage::{TokenStorage, LocalTokenStorage};

let storage = LocalTokenStorage::new();

// 토큰 저장
if let Err(e) = storage.set_token("eyJ...") {
    log::error!("Failed to save token: {}", e);
    // fallback: 세션 내 메모리에만 저장
}

// 토큰 읽기
if let Some(token) = storage.get_token() {
    log::info!("Token loaded from localStorage");
} else {
    log::info!("No token found, user is anonymous");
}

// 토큰 삭제 (로그아웃)
let _ = storage.remove_token();
```

**localStorage 비활성화 시 폴백 전략**:
1. `get_token()` → None 반환 → Anonymous 상태로 전환
2. 공용 세션 생성/수정은 계속 가능 (인증 불필요)
3. UI에 "브라우저 설정에서 localStorage를 활성화하세요" 메시지 표시

---

## 4. gloo-net 인증 헤더 관리

### Decision
**gloo-net Request에 Authorization 헤더를 추가하는 헬퍼 함수 구현, 401/403 응답 감지**

### Rationale
- **기존 프로젝트 일관성**: 프로젝트가 이미 gloo-net 사용 중
- **WASM 호환**: gloo-net은 WASM 환경에 최적화됨
- **간결한 API**: Request::get(url).header("Authorization", value) 패턴
- **에러 처리**: response.status() 및 response.json()으로 구조화된 에러 추출

### Alternatives Considered
- **reqwest (wasm)**: 더 무거운 의존성, gloo-net으로 충분
- **web-sys fetch 직접 사용**: 저수준 API, 보일러플레이트 코드 증가
- **GraphQL 클라이언트**: REST API이므로 불필요

### Implementation Details

**헬퍼 함수 (src/api/mod.rs)**:
```rust
use gloo_net::http::{Request, Response};
use crate::auth::storage::{TokenStorage, LocalTokenStorage};

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
    pub fn get(&self, url: &str) -> Request {
        let mut req = Request::get(url);
        
        if let Some(token) = self.storage.get_token() {
            req = req.header("Authorization", &format!("Bearer {}", token));
        }
        
        req
    }
    
    /// POST 요청 (인증 필요)
    pub fn post(&self, url: &str) -> Request {
        let mut req = Request::post(url);
        
        if let Some(token) = self.storage.get_token() {
            req = req.header("Authorization", &format!("Bearer {}", token));
        }
        
        req
    }
    
    /// PUT 요청 (인증 필요)
    pub fn put(&self, url: &str) -> Request {
        let mut req = Request::put(url);
        
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
            Err("인증이 만료되었습니다. 다시 로그인하세요".to_string())
        }
        403 => {
            log::warn!("403 Forbidden - insufficient permissions");
            Err("권한이 없습니다".to_string())
        }
        404 => {
            Err("세션을 찾을 수 없습니다".to_string())
        }
        409 => {
            Err("이미 존재하는 세션 ID입니다".to_string())
        }
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
```

**사용 예시 (src/api/user_session.rs)**:
```rust
use crate::api::{AuthenticatedRequest, handle_response};

/// 유저별 세션 생성
pub async fn create_user_session(
    user_id: &str,
    session_id: &str,
    template: &str,
    args: serde_json::Value,
) -> Result<(), String> {
    let url = format!("/api/user/{}/session", user_id);
    let body = serde_json::json!({
        "session_id": session_id,
        "template": template,
        "args": args,
    });
    
    let auth_req = AuthenticatedRequest::new();
    let response = auth_req.post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    handle_response(response).await?;
    Ok(())
}
```

**타임아웃 설정 (선택적)**:
```rust
// gloo-net은 기본 타임아웃 제공하지 않음
// 필요 시 web-sys AbortController 사용
use web_sys::AbortController;

let abort_controller = AbortController::new().ok();
let signal = abort_controller.as_ref().map(|c| c.signal());

// 30초 후 요청 취소
if let Some(controller) = abort_controller {
    gloo_timers::callback::Timeout::new(30_000, move || {
        controller.abort();
    }).forget();
}
```

---

## 5. Yew 컴포넌트 상태 관리

### Decision
**use_state 기반 로컬 상태 + AuthContext (use_context) 기반 전역 로그인 상태 관리**

### Rationale
- **로컬 상태**: 폼 입력, API 로딩 상태 등 컴포넌트별 관심사는 use_state로 관리
- **전역 상태**: 로그인 상태(AuthState)는 여러 컴포넌트에서 필요하므로 Context API 사용
- **단순성**: Redux/Zustand 같은 복잡한 상태 관리 라이브러리 불필요
- **Yew 네이티브**: use_context는 Yew 내장 기능, 추가 의존성 없음

### Alternatives Considered
- **props drilling**: 모든 컴포넌트에 AuthState를 props로 전달 → 보일러플레이트 과다
- **use_reducer**: 복잡한 상태 전이가 없어 use_state로 충분
- **외부 상태 라이브러리 (yewdux)**: 프로젝트 규모가 작아 과도한 추상화

### Implementation Details

**AuthContext 정의 (src/auth/mod.rs)**:
```rust
use yew::prelude::*;
use crate::auth::storage::{TokenStorage, LocalTokenStorage};
use crate::auth::token::Claims;

#[derive(Debug, Clone, PartialEq)]
pub enum AuthState {
    Anonymous,
    Authenticated { user_id: String },
}

#[derive(Clone, PartialEq)]
pub struct AuthContext {
    pub state: AuthState,
    pub login: Callback<String>,      // 토큰 저장 및 상태 갱신
    pub logout: Callback<()>,         // 토큰 삭제 및 Anonymous로 전환
}

#[derive(Properties, PartialEq)]
pub struct AuthProviderProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(AuthProvider)]
pub fn auth_provider(props: &AuthProviderProps) -> Html {
    let storage = LocalTokenStorage::new();
    
    // 초기 상태: localStorage에서 토큰 로드
    let state = use_state(|| {
        if let Some(token) = storage.get_token() {
            if let Ok(claims) = Claims::from_token(&token) {
                if !claims.is_expired() {
                    return AuthState::Authenticated {
                        user_id: claims.user_id().to_string(),
                    };
                } else {
                    // 만료된 토큰 삭제
                    let _ = storage.remove_token();
                }
            }
        }
        AuthState::Anonymous
    });
    
    let login = {
        let state = state.clone();
        let storage = storage.clone();
        Callback::from(move |token: String| {
            if let Ok(claims) = Claims::from_token(&token) {
                if !claims.is_expired() {
                    let _ = storage.set_token(&token);
                    state.set(AuthState::Authenticated {
                        user_id: claims.user_id().to_string(),
                    });
                }
            }
        })
    };
    
    let logout = {
        let state = state.clone();
        let storage = storage.clone();
        Callback::from(move |_| {
            let _ = storage.remove_token();
            state.set(AuthState::Anonymous);
        })
    };
    
    let ctx = AuthContext {
        state: (*state).clone(),
        login,
        logout,
    };
    
    html! {
        <ContextProvider<AuthContext> context={ctx}>
            {props.children.clone()}
        </ContextProvider<AuthContext>>
    }
}
```

**App에서 Provider 래핑 (src/lib.rs)**:
```rust
use crate::auth::AuthProvider;

#[function_component(App)]
fn app() -> Html {
    html! {
        <AuthProvider>
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </AuthProvider>
    }
}
```

**컴포넌트에서 AuthContext 사용**:
```rust
use yew::prelude::*;
use crate::auth::{AuthContext, AuthState};

#[function_component(Header)]
pub fn header() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    
    match &auth.state {
        AuthState::Anonymous => html! {
            <div class="header">
                <span>{"비로그인 상태"}</span>
                <Link<Route> to={Route::Home}>{"로그인"}</Link<Route>>
            </div>
        },
        AuthState::Authenticated { user_id } => {
            let on_logout = {
                let logout = auth.logout.clone();
                Callback::from(move |_| logout.emit(()))
            };
            
            html! {
                <div class="header">
                    <span>{format!("안녕하세요, {}", user_id)}</span>
                    <button onclick={on_logout}>{"로그아웃"}</button>
                </div>
            }
        }
    }
}
```

**비동기 작업 패턴 (spawn_local)**:
```rust
use wasm_bindgen_futures::spawn_local;

#[function_component(LoginForm)]
pub fn login_form() -> Html {
    let auth = use_context::<AuthContext>().unwrap();
    let loading = use_state(|| false);
    let error = use_state(|| None::<String>);
    
    let on_submit = {
        let loading = loading.clone();
        let error = error.clone();
        let login = auth.login.clone();
        
        Callback::from(move |user_id: String| {
            loading.set(true);
            error.set(None);
            
            let loading = loading.clone();
            let error = error.clone();
            let login = login.clone();
            
            spawn_local(async move {
                match fetch_token(&user_id).await {
                    Ok(token) => {
                        login.emit(token);
                    }
                    Err(e) => {
                        error.set(Some(e));
                    }
                }
                loading.set(false);
            });
        })
    };
    
    // ... UI 렌더링
}
```

---

## 요약

| 항목 | 선택 기술 | 핵심 이점 |
|-----|----------|----------|
| **라우팅** | yew-router 0.18 | 타입 안전, 동적 파라미터, History API |
| **JWT 디코딩** | base64 0.22 + serde_json | 경량, WASM 번들 절감 |
| **토큰 저장** | web-sys Storage + 커스텀 래퍼 | 에러 핸들링, 테스트 가능성 |
| **HTTP 클라이언트** | gloo-net + 인증 헬퍼 | 간결한 API, 401/403 자동 처리 |
| **상태 관리** | use_state + use_context | 단순성, Yew 네이티브 |

**다음 단계**: data-model.md와 contracts/ 생성
