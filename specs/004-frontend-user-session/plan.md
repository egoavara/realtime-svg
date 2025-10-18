# Implementation Plan: 프론트엔드 유저 세션 UI 통합

**Branch**: `004-frontend-user-session` | **Date**: 2025-10-18 | **Spec**: [spec.md](./spec.md)  
**Input**: Feature specification from `/workspaces/realtime-svg/specs/004-frontend-user-session/spec.md`

## Summary

프론트엔드에서 백엔드의 유저별 세션 기능을 사용할 수 있도록 **yew-router 기반 SPA 라우팅**, **localStorage 기반 JWT 토큰 관리**, **타입 안전한 HTTP 클라이언트**를 구현합니다. yew-router를 도입하여 `/session/{session_id}` (공용)와 `/session/{user_id}/{session_id}` (유저별)를 URL 패턴으로 자동 구분하고, base64 크레이트로 JWT 디코딩하여 클라이언트 측에서 user_id를 추출합니다.

## Technical Context

**Language/Version**: Rust 2021 Edition, WASM target (wasm32-unknown-unknown)  
**Primary Dependencies**: 
- `yew` 0.21 (UI framework, CSR 모드)
- `yew-router` 0.18 (SPA routing, BrowserRouter)
- `gloo-net` 0.4 (HTTP client, WASM 호환)
- `base64` 0.22 (JWT payload 디코딩)
- `web-sys` 0.3 (localStorage 바인딩, Storage feature)
- `serde`/`serde_json` (직렬화/역직렬화)

**Storage**: localStorage (클라이언트 측 JWT 토큰 저장)  
**Testing**: `wasm-bindgen-test` (WASM 환경 단위 테스트)  
**Target Platform**: WASM32-unknown-unknown, 모던 브라우저 (Chrome 90+, Firefox 88+, Safari 14+)  
**Project Type**: Web (Frontend WASM)  
**Performance Goals**: 
- 페이지 전환 100ms 이하
- localStorage 읽기/쓰기 10ms 이하
- JWT 디코딩 1ms 이하
- WASM 번들 크기 500KB 이하

**Constraints**: 
- localStorage 사용 불가 시 공용 세션 폴백
- WASM 실행 실패 시 에러 페이지 표시
- 브라우저 히스토리 API 필수

**Scale/Scope**: 
- 4개 라우트 (Home, PublicSession, UserSession, MySessions)
- ~10개 Yew 컴포넌트
- ~5개 API 엔드포인트 통합

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

✅ **I. Workspace Modularity**: 프론트엔드 크레이트는 `crates/frontend`에 독립적으로 존재, `common` 크레이트와 타입 공유 (SessionData 등)

✅ **II. Contract-First API Design**: 백엔드 API 계약(spec 002의 user-session-api.yaml)을 준수하는 HTTP 클라이언트 구현

✅ **III. Template-Based SVG Rendering**: 프론트엔드는 렌더링 로직 없음, 백엔드 스트림 소비만 수행

✅ **IV. Testing Discipline**: 
- Unit tests: JWT 디코딩, localStorage wrapper, 라우터 파싱 로직
- Integration tests: API 호출 모킹, 컴포넌트 렌더링 검증

✅ **V. Observability & Debugging**: `wasm-logger` 사용, 브라우저 콘솔에 구조화된 로그 출력

✅ **VI. Simplicity & Incremental Delivery**: 
- P0: JWT 토큰 발급 UI (핵심 전제)
- P1: 유저별 세션 생성/수정 (핵심 기능)
- P2: 세션 목록 조회, 로그아웃 (편의 기능)

**GATE: PASS** ✅ - 모든 constitution 원칙 준수

### Post-Design Re-check

*Phase 1 설계 완료 후 재검증*

✅ **I. Workspace Modularity**: 
- 프론트엔드는 `crates/frontend/src` 내부에 모듈 구조 (`auth/`, `components/`, `api/`) 명확히 분리
- `common` 크레이트에서 SessionData 타입 재사용 (변경 없음)

✅ **II. Contract-First API Design**: 
- `contracts/frontend-api.md`에 모든 API 엔드포인트 계약 명시
- spec 002의 `user-session-api.yaml` 준수 확인
- 요청/응답 스키마를 Rust 구조체로 타입 안전하게 정의

✅ **III. Template-Based SVG Rendering**: 
- 프론트엔드는 렌더링 수행하지 않음, 백엔드 스트림만 소비
- 위반 없음

✅ **IV. Testing Discipline**: 
- `data-model.md`에 단위 테스트 대상 명시 (JWT 디코딩, localStorage wrapper)
- `wasm-bindgen-test` 사용 계획 수립
- MockTokenStorage 패턴으로 테스트 가능성 확보

✅ **V. Observability & Debugging**: 
- `research.md`에 wasm-logger 사용 계획 명시
- 브라우저 콘솔 로그 출력 (log::info, log::warn, log::error)

✅ **VI. Simplicity & Incremental Delivery**: 
- P0 → P1 → P2 우선순위 명확 (spec.md 및 plan.md)
- use_state + use_context 기반 단순한 상태 관리 (Redux 등 복잡한 라이브러리 배제)

**GATE: PASS** ✅ - Phase 1 설계가 constitution 원칙 준수 확인 완료

## Project Structure

### Documentation (this feature)

```
specs/004-frontend-user-session/
├── plan.md              # This file
├── research.md          # Phase 0 output (yew-router, JWT, localStorage 연구)
├── data-model.md        # Phase 1 output (클라이언트 측 엔티티)
├── quickstart.md        # Phase 1 output (로컬 개발 가이드)
└── contracts/           # Phase 1 output (API 호출 계약)
    └── frontend-api.md
```

### Source Code (repository root)

```
crates/frontend/
├── src/
│   ├── lib.rs                    # 기존 (Yew 앱 엔트리, BrowserRouter 추가)
│   ├── main.rs                   # 기존 (WASM 빌드 타겟)
│   ├── routes.rs                 # 신규 (yew-router Route enum 정의)
│   ├── auth/
│   │   ├── mod.rs                # 신규 (JWT 토큰 관리 모듈)
│   │   ├── token.rs              # 신규 (JWT 디코딩, Claims 추출)
│   │   └── storage.rs            # 신규 (localStorage wrapper)
│   ├── components/
│   │   ├── mod.rs                # 신규 (컴포넌트 모듈 루트)
│   │   ├── login_form.rs         # 신규 (토큰 발급 UI)
│   │   ├── session_form.rs       # 리팩토링 (기존 CreatePage → 공용/유저 모드 구분)
│   │   ├── session_detail.rs     # 리팩토링 (기존 DetailPage → 라우트 파라미터 활용)
│   │   ├── session_list.rs       # 신규 (세션 목록 컴포넌트)
│   │   └── header.rs             # 신규 (로그인 상태 표시 헤더)
│   ├── api/
│   │   ├── mod.rs                # 신규 (API 클라이언트 모듈)
│   │   ├── auth.rs               # 신규 (POST /api/auth/token)
│   │   ├── user_session.rs       # 신규 (유저별 세션 CRUD API)
│   │   └── public_session.rs     # 리팩토링 (기존 API 호출 로직 분리)
│   ├── types.rs                  # 신규 (클라이언트 측 타입 정의)
│   └── utils.rs                  # 신규 (공통 헬퍼 함수)
├── Cargo.toml                    # 수정 (yew-router, base64, web-sys Storage 추가)
└── index.html                    # 기존 (변경 없음)
```

**Structure Decision**: Cargo workspace의 `frontend` 크레이트 내부에 기능별 모듈 구조를 채택. `auth`, `components`, `api` 모듈로 관심사 분리하여 유지보수성 향상. 기존 lib.rs의 모놀리식 코드를 모듈로 분리하여 테스트 가능성과 재사용성을 개선.

## Complexity Tracking

*No constitution violations - this section is empty.*

---

## Phase 0: Research & Technology Decisions

**Status**: ✅ Complete  
**Output**: [research.md](./research.md)

### Research Topics

1. **yew-router 0.18 통합 방법**
   - Route enum 정의 (`#[derive(Routable)]`)
   - BrowserRouter 설정
   - 동적 경로 파라미터 추출 (`/session/:user_id/:session_id`)
   - Link 컴포넌트 및 use_navigator hook 사용법

2. **JWT 디코딩 (클라이언트 측)**
   - base64 크레이트로 payload 디코딩
   - sub claim 추출 로직
   - 만료 시간 체크 (exp claim)
   - 서명 검증 불필요 (백엔드에서 검증)

3. **localStorage 안전한 래핑**
   - web-sys Storage API 바인딩
   - Option 처리 (localStorage 비활성화 시)
   - 에러 핸들링 (QuotaExceededError 등)
   - 타입 안전한 get/set/remove 함수

4. **gloo-net 인증 헤더 관리**
   - Authorization: Bearer 헤더 추가 패턴
   - 401/403 응답 감지 및 처리
   - JSON 에러 본문 파싱
   - 타임아웃 설정 (30초)

5. **Yew 컴포넌트 상태 관리**
   - use_state vs use_reducer 선택 기준
   - 비동기 작업 spawn_local 패턴
   - 컴포넌트 간 상태 전파 (Context API vs props)

**Deliverable**: `research.md` 파일 생성 (각 주제별 Decision, Rationale, Alternatives Considered, Implementation Details)

---

## Phase 1: Design & Contracts

**Status**: ✅ Complete  
**Outputs**: 
- ✅ `data-model.md`
- ✅ `contracts/frontend-api.md`
- ✅ `quickstart.md`
- ✅ Agent context 업데이트 (AGENTS.md)

### 1. Data Model Design

**클라이언트 측 엔티티 정의**:

```rust
// 로그인 상태
enum AuthState {
    Anonymous,                           // 비로그인
    Authenticated { user_id: String },   // 로그인 (토큰 있음)
}

// 라우트 정의
#[derive(Debug, Clone, Copy, PartialEq, Routable)]
enum Route {
    #[at("/")]
    Home,
    #[at("/session/:session_id")]
    PublicSession { session_id: String },
    #[at("/session/:user_id/:session_id")]
    UserSession { user_id: String, session_id: String },
    #[at("/my-sessions")]
    MySessions,
    #[not_found]
    #[at("/404")]
    NotFound,
}

// API 요청 상태
enum ApiState<T> {
    Idle,
    Loading,
    Success(T),
    Error(String),
}

// JWT Claims
struct Claims {
    sub: String,     // user_id
    exp: u64,        // 만료 시간 (Unix timestamp)
    iat: u64,        // 발급 시간
    iss: String,     // 발급자
}

// 세션 목록 항목
struct SessionListItem {
    session_id: String,
    created_at: String,
    ttl: u64,
}
```

**상태 전이도**:
```
[App 시작] --> [localStorage 조회] --> [토큰 있음?]
                                          ├─ Yes: Authenticated(user_id)
                                          └─ No: Anonymous

[토큰 발급 요청] --> [POST /api/auth/token] --> [localStorage 저장] --> Authenticated
[401 응답] --> [localStorage 삭제] --> Anonymous
[로그아웃 버튼] --> [localStorage 삭제] --> Anonymous
```

### 2. API Contracts

**클라이언트가 호출하는 엔드포인트**:

| 엔드포인트 | 메서드 | 인증 | 요청 본문 | 응답 |
|-----------|--------|------|-----------|------|
| `/api/auth/token` | POST | ❌ | `{"user_id": "alice"}` | `{"token": "eyJ..."}` |
| `/api/user/{user_id}/session` | POST | ✅ | `{session_id, template, args?, expire?}` | `{user_id, session_id}` |
| `/api/user/{user_id}/session/{session_id}` | PUT | ✅ | `{args}` | 204 No Content |
| `/api/user/{user_id}/session/{session_id}` | GET | ❌ | - | `{template, args}` |
| `/api/user/{user_id}/sessions` | GET | ✅ | - | `{sessions: [{session_id, created_at, ttl}]}` |
| `/api/session` | POST | ❌ | `{session_id, template, args?, expire?}` | `{session_id}` |
| `/api/session/{session_id}` | PUT | ❌ | `{args}` | 204 No Content |
| `/api/session/{session_id}` | GET | ❌ | - | `{template, args}` |

**에러 응답 형식**:
```json
{
  "error": "Unauthorized | Forbidden | SessionNotFound | ..."
}
```

### 3. Quickstart Guide

**로컬 개발 환경 설정**:
```bash
# 1. Trunk 설치
cargo install trunk

# 2. WASM 타겟 추가
rustup target add wasm32-unknown-unknown

# 3. 백엔드 실행 (터미널 1)
cd /workspaces/realtime-svg/crates/backend
cargo run

# 4. 프론트엔드 개발 서버 실행 (터미널 2)
cd /workspaces/realtime-svg/crates/frontend
trunk serve --open

# 5. 브라우저에서 http://127.0.0.1:8080 접속
```

**테스트 시나리오**:
1. 홈 페이지에서 "user_id: alice" 입력 → 토큰 발급
2. "유저별 세션 만들기" 선택 → session_id: "test-1" 입력 → 생성
3. 파라미터 수정 → 실시간 스트림 갱신 확인
4. "내 세션 목록" 클릭 → "test-1" 표시 확인
5. 로그아웃 → Anonymous 상태 확인
6. 공용 세션 생성 → 토큰 없이 작동 확인

### 4. Agent Context Update

```bash
cd /workspaces/realtime-svg
.specify/scripts/bash/update-agent-context.sh opencode
```

**추가될 기술 스택**:
- yew-router 0.18
- base64 0.22
- web-sys (Storage feature)

---

## Phase 2: Task Breakdown

**Status**: ⏳ Pending (Phase 1 완료 후)  
**Command**: `/speckit.tasks`  
**Output**: `tasks.md`

예상 작업 카테고리:
- **P0 Tasks**: JWT 토큰 발급 UI, localStorage wrapper, base64 JWT 디코딩
- **P1 Tasks**: yew-router 통합, 유저별 세션 생성/수정 UI, 공용 세션 호환성
- **P2 Tasks**: 세션 목록 조회 UI, 로그아웃 기능

---

## Next Steps

1. ✅ Plan.md 작성 완료
2. 🔄 Research.md 생성 (Phase 0)
3. ⏳ Data-model.md 생성 (Phase 1)
4. ⏳ Contracts 생성 (Phase 1)
5. ⏳ Quickstart.md 생성 (Phase 1)
6. ⏳ Agent context 업데이트 (Phase 1)
7. ⏳ Tasks.md 생성 (`/speckit.tasks` 명령어로)

**Current Phase**: Phase 2 - Task Breakdown  
**Blocking**: None  
**Ready for**: `/speckit.tasks` 명령어 실행
