# Implementation Plan: 사용자별 세션 인증

**Branch**: `002-user-session-auth` | **Date**: 2025-10-17 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-user-session-auth/spec.md`

## Summary

사용자별 세션 인증 기능을 구현하여 특정 사용자만 자신의 세션을 수정할 수 있도록 한다. 시스템은 자체 JWT 발급 시스템을 구축하고, JWK를 JWK 형식으로 Redis에 저장하며 메모리에 캐싱하여 성능을 최적화한다. `/stream/{user_id}/{session_id}` 형태의 사용자별 스트림 엔드포인트를 추가하되, 기존 공용 세션 API의 하위 호환성을 유지한다.

## Technical Context

**Language/Version**: Rust 2021  
**Primary Dependencies**: axum (HTTP), tokio (async), redis (session/pubsub), jsonwebtoken 10.x (JWT + Jwk), serde_json (serialization), tera (template)  
**Storage**: Redis (세션 데이터, JWK 키 쌍, pubsub)  
**Testing**: cargo test (unit), integration tests (API contracts)  
**Target Platform**: Linux server, WASM (frontend)  
**Project Type**: Web (multi-crate workspace: backend, common, frontend)  
**Performance Goals**: JWT 검증 50ms 이하 (메모리 캐시), JWK 로드 5초 이내, 토큰 발급 1초 이내  
**Constraints**: 메모리 캐시로 Redis 조회 0회 (JWT 검증 시), 기존 공용 세션 API 완전 호환성 유지  
**Scale/Scope**: 사용자당 100+ 세션, 동시 접속 1000+ 사용자

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Workspace Modularity ✅
- **Status**: PASS
- JWT 관련 타입과 로직을 `common` 크레이트에 정의
- 백엔드는 HTTP 핸들러만 담당
- 프론트엔드는 UI 로직만 담당
- 각 크레이트 독립적으로 테스트 가능

### II. Contract-First API Design ✅
- **Status**: PASS
- 새로운 API 엔드포인트 명세 필요:
  - `POST /api/auth/token` - JWT 발급
  - `GET /api/auth/jwks` - 공개 키 조회
  - `POST /api/user/{user_id}/session` - 사용자 세션 생성
  - `PUT /api/user/{user_id}/session/{session_id}` - 사용자 세션 수정
  - `GET /api/user/{user_id}/sessions` - 세션 목록 조회
  - `GET /stream/{user_id}/{session_id}` - 사용자별 스트림
- Phase 1에서 OpenAPI 스키마 생성 예정

### III. Template-Based SVG Rendering ✅
- **Status**: PASS
- 기존 Tera 템플릿 엔진 사용 유지
- 세션 데이터 구조 확장 (소유자 정보 추가)
- 템플릿 렌더링 로직 변경 없음

### IV. Testing Discipline ✅
- **Status**: PASS (Phase 1에서 구현)
- Unit tests:
  - JWT 생성/검증 로직
  - JWK 생성/직렬화
  - 권한 검증 로직
- Integration tests:
  - JWT 발급 API
  - 사용자 세션 CRUD API
  - 권한 검증 시나리오
- Contract tests:
  - JWT 토큰 구조 검증
  - JWK 형식 검증

### V. Observability & Debugging ✅
- **Status**: PASS
- 추가 로깅 포인트:
  - JWT 발급 요청 (user_id)
  - JWT 검증 실패 (원인 구분: 만료/서명/형식)
  - JWK 생성 및 로드
  - 권한 검증 실패 (403/401 구분)
  - 메모리 캐시 히트/미스

### VI. Simplicity & Incremental Delivery ✅
- **Status**: PASS
- P0: JWT 발급 시스템 (기반 인프라)
- P1: 사용자 세션 생성 및 권한 검증
- P2: 기존 API 호환성 유지
- P3: 세션 목록 조회
- 각 우선순위별 독립 테스트 가능

## Project Structure

### Documentation (this feature)

```
specs/002-user-session-auth/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── auth-api.yaml    # JWT 발급/검증 API
│   └── user-session-api.yaml  # 사용자 세션 API
└── tasks.md             # Phase 2 output (NOT created by /speckit.plan)
```

### Source Code (repository root)

```
crates/
├── common/
│   ├── src/
│   │   ├── jwt.rs          # JWT 생성/검증 로직 (신규)
│   │   ├── jwk.rs          # Jwk 객체 Redis 저장/로드/메모리캐시 (신규)
│   │   ├── auth.rs         # 인증/권한 검증 헬퍼 (신규)
│   │   ├── session_data.rs # SessionData 확장 (owner 필드 추가)
│   │   ├── state.rs        # AppState 확장 (JWK 캐시 추가)
│   │   └── errors.rs       # 인증 오류 타입 추가
│   └── Cargo.toml          # jsonwebtoken, base64 추가
│
├── backend/
│   ├── src/
│   │   ├── route/
│   │   │   ├── api/
│   │   │   │   ├── auth/          # 신규 디렉토리
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── http_post_token.rs      # POST /api/auth/token
│   │   │   │   │   └── http_get_jwks.rs        # GET /.well-known/jwks.json
│   │   │   │   ├── user/          # 신규 디렉토리
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── http_post_session.rs    # POST /api/user/{user_id}/session
│   │   │   │   │   ├── http_put_session.rs     # PUT /api/user/{user_id}/session/{session_id}
│   │   │   │   │   └── http_get_sessions.rs    # GET /api/user/{user_id}/sessions
│   │   │   │   └── mod.rs         # auth, user 라우터 추가
│   │   │   └── stream/
│   │   │       └── http_get_user_stream.rs      # GET /stream/{user_id}/{session_id}
│   │   └── middleware/
│   │       └── jwt_auth.rs        # JWT 검증 미들웨어 (신규)
│   └── tests/
│       ├── jwt_flow_test.rs       # JWT 발급/검증 통합 테스트
│       └── user_session_test.rs   # 사용자 세션 권한 테스트
│
└── frontend/
    └── src/
        └── lib.rs                  # JWT 토큰 발급 UI (선택적)
```

**Structure Decision**: Multi-crate workspace 유지. JWT/인증 관련 비즈니스 로직은 `common` 크레이트에 배치하여 재사용성과 테스트 용이성 확보. 백엔드는 HTTP 핸들러와 미들웨어만 담당.

## Complexity Tracking

*No constitution violations - all gates passed*

## Phase 0: Research & Decision Log

### Research Topics

1. **JWK 형식 및 Redis 저장 방법**
   - jsonwebtoken 라이브러리의 Jwk 객체 사용
   - Jwk 객체를 serde_json으로 직렬화하여 Redis 저장
   - Redis SET NX로 원자적 생성 방법

2. **JWT 메모리 캐싱 전략**
   - Rust에서 전역 메모리 캐시 패턴 (lazy_static, once_cell)
   - 애플리케이션 수명 동안 JWK 캐시 유지 방법
   - 멀티스레드 환경에서 안전한 캐시 접근

3. **JWT 라이브러리 선택**
   - `jsonwebtoken` vs 다른 JWT 라이브러리
   - RSA-2048 서명 지원 여부
   - JWK 형식 지원 여부

4. **사용자별 세션 Redis 키 설계**
   - 네임스페이스 분리 전략 (`user:{user_id}:session:{session_id}`)
   - 세션 목록 조회를 위한 패턴 매칭 (`SCAN` vs `KEYS`)
   - TTL 관리 및 자동 만료

5. **권한 검증 미들웨어 구현**
   - axum 미들웨어 패턴
   - JWT 추출 및 검증 위치
   - 권한 실패 시 에러 응답 처리

### Research Execution

*Phase 0에서 위 주제들에 대한 연구 결과를 research.md에 작성*

## Phase 1: Design Artifacts ✅

**완료된 파일들**:

1. ✅ **research.md**: JWT/JWK/Redis 연구 결과 (Phase 0)
2. ✅ **data-model.md**: JWK, JWT Token, User Session 엔티티 상세 설계
3. ✅ **contracts/auth-api.yaml**: JWT 발급/검증 API OpenAPI 스펙
4. ✅ **contracts/user-session-api.yaml**: 사용자 세션 API OpenAPI 스펙
5. ✅ **quickstart.md**: JWT 발급 → 세션 생성 → 스트림 접근 빠른 시작 가이드
6. ✅ **AGENTS.md 업데이트**: opencode 컨텍스트에 JWT/JWK 기술 스택 추가

## Post-Design Constitution Re-Check

### I. Workspace Modularity ✅
- **Status**: PASS
- JWT 로직을 `common/src/jwt/` 모듈로 구성
- 백엔드는 `route/api/auth/`, `route/api/user/`로 HTTP 레이어만 담당
- 각 크레이트 독립 테스트 가능한 구조 유지

### II. Contract-First API Design ✅
- **Status**: PASS
- OpenAPI 3.0 스펙 완성:
  - `contracts/auth-api.yaml`: JWT 발급/JWKS 조회
  - `contracts/user-session-api.yaml`: 사용자 세션 CRUD
- 모든 엔드포인트 요청/응답 스키마 정의 완료
- 에러 코드 및 시나리오 명시 (401/403/404/409)

### III. Template-Based SVG Rendering ✅
- **Status**: PASS
- 기존 Tera 템플릿 엔진 유지
- SessionData 구조체에 `owner: Option<String>` 필드만 추가
- 템플릿 렌더링 로직 변경 없음

### IV. Testing Discipline ✅
- **Status**: PASS (설계 완료, 구현 예정)
- Unit tests 계획:
  - `common/src/jwt/`: JWT 생성/검증, JWK 변환
  - `common/src/auth.rs`: Extractor, 권한 검증
- Integration tests 계획:
  - JWT 발급 → 세션 생성 → 업데이트 → 스트림 플로우
  - 권한 거부 시나리오 (401/403)
- Contract tests 계획:
  - JWT 토큰 구조 검증
  - OpenAPI 스펙 준수 검증

### V. Observability & Debugging ✅
- **Status**: PASS
- 로깅 포인트 설계:
  - JWT 발급: `tracing::info!("Issued token for user: {}", user_id)`
  - JWT 검증 실패: `tracing::warn!("JWT verification failed: {}", error)`
  - JWK 초기화: `tracing::info!("JWK cached in memory")`
  - 권한 거부: `tracing::warn!("User {} attempted to access user {} session", auth_user, url_user)`

### VI. Simplicity & Incremental Delivery ✅
- **Status**: PASS
- Phase 구성:
  - Phase 0 (완료): 연구 및 기술 선택
  - Phase 1 (완료): 데이터 모델 및 API 설계
  - Phase 2 (예정): 태스크 분해 및 우선순위 (P0 → P1 → P2 → P3)
- 최소 기능부터 시작 (JWK 생성 → JWT 발급 → 세션 인증)

## Next Steps

1. ✅ **Phase 0 완료**: 연구 결과 research.md 생성
2. ✅ **Phase 1 완료**: 데이터 모델 및 API contracts 생성
3. ✅ **Agent Context 업데이트 완료**: AGENTS.md에 JWT 기술 스택 추가
4. **Phase 2 진행**: `/speckit.tasks` 명령으로 구현 태스크 목록 생성
