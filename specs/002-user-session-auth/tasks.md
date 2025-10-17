# Tasks: 사용자별 세션 인증

**Input**: Design documents from `/specs/002-user-session-auth/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Testing Discipline (Constitution IV) - Unit, Integration, Contract tests 포함

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US4, US1, US2, US3, US5)
- Include exact file paths in descriptions

## Path Conventions
- **Multi-crate workspace**: `crates/common/src/`, `crates/backend/src/`, `crates/frontend/src/`
- Constitution I: Workspace modularity - JWT logic in common, HTTP handlers in backend

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and dependency setup

- [X] T001 Add jsonwebtoken 10.x dependency to crates/common/Cargo.toml
- [X] T002 [P] Add tokio::sync dependency to crates/common/Cargo.toml for OnceCell
- [X] T003 [P] Add chrono dependency to crates/common/Cargo.toml for timestamps
- [X] T004 [P] Create crates/common/src/jwt.rs module file
- [X] T005 [P] Create crates/common/src/jwk.rs module file
- [X] T006 [P] Create crates/common/src/auth.rs module file
- [X] T007 [P] Create crates/backend/src/route/api/auth/ directory
- [X] T008 [P] Create crates/backend/src/route/api/user/ directory

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### JWK Infrastructure (FR-001 to FR-004, FR-008)

- [X] T009 Implement Jwk generation and Redis storage in crates/common/src/jwk.rs
- [X] T010 Implement JwkCache struct with OnceCell in crates/common/src/jwk.rs
- [X] T011 Implement atomic JWK creation with SET NX in crates/common/src/jwk.rs
- [X] T012 Add JwkCache to AppState in crates/common/src/state.rs
- [X] T013 Implement JWK initialization on app startup in crates/common/src/state.rs

### JWT Core Logic (FR-006, FR-007, FR-010)

- [X] T014 Define Claims struct in crates/common/src/jwt.rs
- [X] T015 Implement JWT token creation with EncodingKey in crates/common/src/jwt.rs
- [X] T016 Implement JWT token verification with DecodingKey in crates/common/src/jwt.rs
- [X] T017 Add logging for JWT operations in crates/common/src/jwt.rs

### Error Types (FR-015, FR-016, FR-026)

- [X] T018 Add Unauthorized and Forbidden variants to ApiError in crates/common/src/errors.rs
- [X] T019 Implement IntoResponse for auth errors with 401/403 status codes in crates/common/src/errors.rs

### SessionData Extension (FR-013)

- [X] T020 Add owner: Option<String> field to SessionData in crates/common/src/session_data.rs
- [X] T021 Update SessionData serialization/deserialization logic

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 4 - JWT 토큰 발급 (Priority: P0) 🎯 Foundation

**Goal**: 사용자가 JWT 토큰을 발급받고, 시스템이 JWK를 Redis에 저장하여 토큰 검증 인프라 제공

**Independent Test**: user_id로 토큰 발급 요청 → 토큰 수신 → 토큰으로 인증 API 호출 → 정상 처리

### Unit Tests for US4

- [X] T022 [P] [US4] Unit test for JWK generation in crates/common/src/jwk.rs
- [X] T023 [P] [US4] Unit test for JWT token creation in crates/common/src/jwt.rs
- [X] T024 [P] [US4] Unit test for JWT token verification in crates/common/src/jwt.rs
- [X] T025 [P] [US4] Unit test for JWT token expiration validation in crates/common/src/jwt.rs

### API Implementation for US4 (FR-005, FR-009)

- [X] T026 [US4] Create crates/backend/src/route/api/auth/mod.rs with router
- [X] T027 [US4] Implement POST /api/auth/token handler in crates/backend/src/route/api/auth/http_post_token.rs
- [X] T028 [US4] Implement GET /.well-known/jwks.json handler in crates/backend/src/route/api/auth/http_get_jwks.rs
- [X] T029 [US4] Return JwkSet with jsonwebtoken::JwkSet in jwks handler
- [X] T030 [US4] Add auth router to main API router in crates/backend/src/route/api/mod.rs
- [X] T031 [US4] Add /.well-known/jwks.json route to root router in crates/backend/src/route/mod.rs

### Integration Tests for US4

- [X] T032 [US4] Integration test: token issuance flow in crates/backend/tests/jwt_flow_test.rs
- [X] T033 [US4] Integration test: expired token rejection in crates/backend/tests/jwt_flow_test.rs
- [X] T034 [US4] Integration test: JWKS endpoint returns valid JwkSet in crates/backend/tests/jwt_flow_test.rs

### Logging for US4 (Constitution V)

- [X] T035 [US4] Add tracing logs for token issuance in http_post_token.rs
- [X] T036 [US4] Add tracing logs for JWK initialization in crates/common/src/jwk.rs

**Checkpoint**: JWT 토큰 발급 및 검증 인프라 완성 - 사용자별 세션 구현 가능

---

## Phase 4: User Story 1 - 사용자별 세션 생성 (Priority: P1) 🎯 MVP

**Goal**: 사용자가 JWT로 인증하여 자신의 세션을 생성하고, 소유권 확보

**Independent Test**: JWT 발급 → 세션 생성 (POST /api/user/{user_id}/session) → 스트림 접근 (GET /stream/{user_id}/{session_id}) → SVG 표시

### JWT Extractor (FR-024, FR-025)

- [X] T037 [US1] Implement AuthenticatedUser extractor in crates/common/src/auth.rs
- [X] T038 [US1] Implement FromRequestParts for AuthenticatedUser with JWT verification
- [X] T039 [US1] Add Authorization header parsing (Bearer token)

### User Session Storage (FR-011, FR-013, FR-017, FR-019)

- [X] T040 [US1] Implement set_user_session in crates/common/src/state.rs
- [X] T041 [US1] Use Redis key pattern user:{user_id}:session:{session_id}
- [X] T042 [US1] Implement get_user_session in crates/common/src/state.rs
- [X] T043 [US1] Use pubsub channel user:{user_id}:session:{session_id}

### API Implementation for US1

- [X] T044 [US1] Create crates/backend/src/route/api/user/mod.rs with router
- [X] T045 [US1] Implement POST /api/user/{user_id}/session in crates/backend/src/route/api/user/http_post_session.rs
- [X] T046 [US1] Validate JWT user_id matches URL user_id (FR-014)
- [X] T047 [US1] Return 403 Forbidden if user_id mismatch (FR-015)
- [X] T048 [US1] Implement GET /stream/{user_id}/{session_id} in crates/backend/src/route/stream/http_get_user_stream.rs
- [X] T049 [US1] Add user router to API router in crates/backend/src/route/api/mod.rs
- [X] T050 [US1] Add user stream route to stream router in crates/backend/src/route/stream/mod.rs

### Unit Tests for US1

- [ ] T051 [P] [US1] Unit test for AuthenticatedUser extractor in crates/common/src/auth.rs
- [X] T052 [P] [US1] Unit test for user session Redis storage in crates/common/src/state.rs
- [ ] T053 [P] [US1] Unit test for user_id validation logic

### Integration Tests for US1

- [X] T054 [US1] Integration test: create user session with valid JWT in crates/backend/tests/user_session_test.rs
- [X] T055 [US1] Integration test: access user stream without auth in crates/backend/tests/user_session_test.rs
- [X] T056 [US1] Integration test: reject session creation with missing JWT in crates/backend/tests/user_session_test.rs
- [X] T057 [US1] Integration test: reject session creation with user_id mismatch in crates/backend/tests/user_session_test.rs

### Logging for US1 (Constitution V)

- [X] T058 [US1] Add tracing logs for session creation with owner info
- [X] T059 [US1] Add tracing logs for stream access (user_id, session_id)

**Checkpoint**: 사용자별 세션 생성 및 스트림 접근 완성 - MVP 배포 가능

---

## Phase 5: User Story 2 - 세션 수정 권한 검증 (Priority: P1)

**Goal**: 세션 소유자만 자신의 세션을 수정 가능, 다른 사용자는 403 반환

**Independent Test**: 사용자 A가 세션 생성 → 사용자 B가 수정 시도 → 403 Forbidden → 사용자 A가 수정 → 성공

### API Implementation for US2 (FR-014, FR-015, FR-016)

- [X] T060 [US2] Implement PUT /api/user/{user_id}/session/{session_id} in crates/backend/src/route/api/user/http_put_session.rs
- [X] T061 [US2] Extract AuthenticatedUser from JWT
- [X] T062 [US2] Validate JWT user_id matches URL user_id
- [X] T063 [US2] Return 403 Forbidden if user_id mismatch
- [X] T064 [US2] Return 401 Unauthorized if JWT invalid/expired
- [X] T065 [US2] Update session args and broadcast via pubsub

### Unit Tests for US2

- [ ] T066 [P] [US2] Unit test for user_id mismatch detection
- [ ] T067 [P] [US2] Unit test for session update with valid owner

### Integration Tests for US2

- [X] T068 [US2] Integration test: owner can update session in crates/backend/tests/user_session_test.rs
- [X] T069 [US2] Integration test: non-owner gets 403 Forbidden in crates/backend/tests/user_session_test.rs
- [X] T070 [US2] Integration test: expired JWT gets 401 Unauthorized in crates/backend/tests/user_session_test.rs
- [X] T071 [US2] Integration test: unauthenticated request gets 401 in crates/backend/tests/user_session_test.rs

### Logging for US2 (Constitution V)

- [X] T072 [US2] Add tracing::warn for 403 Forbidden with user IDs
- [X] T073 [US2] Add tracing::warn for 401 Unauthorized with error reason

**Checkpoint**: 권한 검증 완성 - 사용자별 세션 보안 확보

---

## Phase 6: User Story 3 - 기존 공용 세션 호환성 (Priority: P2)

**Goal**: 기존 /api/session 및 /stream/{session_id} API가 계속 작동, 인증 없이 사용 가능

**Independent Test**: POST /api/session → 공용 세션 생성 → GET /stream/{session_id} → SVG 표시 → 인증 없이 PUT /api/session/{session_id} → 성공

### Compatibility Validation (FR-020, FR-021, FR-022)

- [X] T074 [US3] Verify public session routes in crates/backend/src/route/api/session/mod.rs unchanged
- [X] T075 [US3] Verify public stream route in crates/backend/src/route/stream/mod.rs unchanged
- [X] T076 [US3] Verify public sessions use session:{session_id} Redis key pattern
- [X] T077 [US3] Verify public sessions do not require JWT authentication

### Integration Tests for US3

- [X] T078 [US3] Integration test: create public session without JWT in crates/backend/tests/public_session_compat_test.rs
- [X] T079 [US3] Integration test: update public session without JWT in crates/backend/tests/public_session_compat_test.rs
- [X] T080 [US3] Integration test: access public stream without JWT in crates/backend/tests/public_session_compat_test.rs
- [X] T081 [US3] Integration test: public and user sessions coexist in crates/backend/tests/public_session_compat_test.rs

### Documentation for US3

- [X] T082 [US3] Update quickstart.md to document public vs user session differences

**Checkpoint**: 하위 호환성 확보 - 기존 사용자 영향 없음

---

## Phase 7: User Story 5 - 사용자 세션 목록 조회 (Priority: P3)

**Goal**: 사용자가 자신이 생성한 모든 세션 목록 조회

**Independent Test**: 사용자 A가 3개 세션 생성 → GET /api/user/{user_id}/sessions → 3개 반환 → 사용자 B 세션 미포함

### Session Listing (FR-018)

- [X] T083 [US5] Implement list_user_sessions with SCAN in crates/common/src/state.rs
- [X] T084 [US5] Use Redis pattern user:{user_id}:session:*
- [X] T085 [US5] Implement GET /api/user/{user_id}/sessions in crates/backend/src/route/api/user/http_get_sessions.rs
- [X] T086 [US5] Validate JWT user_id matches URL user_id
- [X] T087 [US5] Return session list with session_id, created_at, TTL

### Unit Tests for US5

- [X] T088 [P] [US5] Unit test for SCAN pattern matching in crates/common/src/state.rs
- [X] T089 [P] [US5] Unit test for session list filtering by user

### Integration Tests for US5

- [X] T090 [US5] Integration test: list user sessions returns only owner's sessions in crates/backend/tests/user_session_test.rs
- [X] T091 [US5] Integration test: empty list for user with no sessions in crates/backend/tests/user_session_test.rs
- [X] T092 [US5] Integration test: list excludes other users' sessions in crates/backend/tests/user_session_test.rs

**Checkpoint**: 세션 목록 조회 완성 - 사용자 편의성 향상

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

### Performance Optimization (SC-003, SC-008)

- [ ] T093 [P] Verify JWK memory cache prevents Redis lookups (log analysis)
- [ ] T094 [P] Benchmark JWT verification latency < 50ms
- [ ] T095 [P] Benchmark token issuance latency < 1s

### Security Hardening

- [ ] T096 [P] Validate JWT signature verification in all auth paths
- [ ] T097 [P] Test token tampering detection
- [ ] T098 [P] Verify no secret leakage in error messages

### Documentation

- [X] T099 [P] Validate all quickstart.md examples work end-to-end
- [X] T100 [P] Add code comments for JWT/JWK logic
- [X] T101 [P] Update README with authentication flow diagram

### Code Quality (Constitution VI)

- [X] T102 Run cargo clippy --workspace -- -D warnings
- [X] T103 Run cargo fmt --check
- [X] T104 Run cargo test --workspace

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Story 4 - JWT 발급 (Phase 3)**: Depends on Foundational - REQUIRED for all other user stories
- **User Story 1 - 세션 생성 (Phase 4)**: Depends on US4 (JWT infrastructure)
- **User Story 2 - 권한 검증 (Phase 5)**: Depends on US1 (session creation)
- **User Story 3 - 호환성 (Phase 6)**: Depends on US1, US2 (validation that new features don't break old ones)
- **User Story 5 - 목록 조회 (Phase 7)**: Depends on US1 (session storage)
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **US4 (P0 - JWT)**: Foundation for all authentication
- **US1 (P1 - 세션 생성)**: Core MVP - can start after US4
- **US2 (P1 - 권한 검증)**: Extends US1 - needs session creation
- **US3 (P2 - 호환성)**: Independent validation - can run in parallel with US2/US5
- **US5 (P3 - 목록 조회)**: Independent feature - can run in parallel with US2/US3

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Unit tests [P] can run in parallel
- Models/storage before services
- Services before API handlers
- Core implementation before integration tests
- Story complete before moving to next priority

### Parallel Opportunities

- Phase 1 (Setup): T002-T008 all [P]
- US4 Unit Tests: T022-T025 all [P]
- US1 Unit Tests: T051-T053 all [P]
- US2 Unit Tests: T066-T067 both [P]
- US5 Unit Tests: T088-T089 both [P]
- Phase 8 (Polish): Most tasks marked [P] can run concurrently

---

## Parallel Example: User Story 4 (JWT)

```bash
# Launch all unit tests for US4 together:
Task T022: "Unit test for JWK generation in crates/common/src/jwk.rs"
Task T023: "Unit test for JWT token creation in crates/common/src/jwt.rs"
Task T024: "Unit test for JWT token verification in crates/common/src/jwt.rs"
Task T025: "Unit test for JWT token expiration validation in crates/common/src/jwt.rs"

# After tests, implement APIs:
Task T027: "Implement POST /api/auth/token handler"
Task T028: "Implement GET /.well-known/jwks.json handler"
```

---

## Implementation Strategy

### MVP First (US4 + US1 Only)

1. Complete Phase 1: Setup (T001-T008)
2. Complete Phase 2: Foundational (T009-T021) - CRITICAL
3. Complete Phase 3: User Story 4 - JWT 발급 (T022-T036)
4. Complete Phase 4: User Story 1 - 세션 생성 (T037-T059)
5. **STOP and VALIDATE**: JWT 발급 → 세션 생성 → 스트림 접근 플로우
6. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational + US4 → JWT infrastructure ready
2. Add US1 → Test independently → Deploy/Demo (MVP with user sessions!)
3. Add US2 → Test security → Deploy/Demo (권한 검증 추가)
4. Add US3 → Test compatibility → Deploy/Demo (기존 사용자 안심)
5. Add US5 → Test listing → Deploy/Demo (편의 기능 추가)
6. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together (T001-T021)
2. Developer A: User Story 4 (JWT) alone (T022-T036) - others blocked
3. Once US4 done:
   - Developer A: User Story 1 (T037-T059)
   - Developer B: User Story 5 (T083-T092) - independent
4. Once US1 done:
   - Developer A: User Story 2 (T060-T073)
   - Developer C: User Story 3 (T074-T082) - validation

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label (US4, US1, US2, US3, US5) maps task to specific user story
- Each user story should be independently completable and testable
- Constitution compliance: Workspace modularity, Contract-first, Template-based SVG, Testing, Observability, Incremental delivery
- Verify tests fail before implementing (TDD)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Performance targets: JWT verify <50ms, token issue <1s, JWK init <5s, 100+ sessions per user

## Total Tasks: 104
- Phase 1 (Setup): 8 tasks
- Phase 2 (Foundational): 13 tasks
- Phase 3 (US4 - JWT): 15 tasks
- Phase 4 (US1 - 세션 생성): 23 tasks
- Phase 5 (US2 - 권한 검증): 14 tasks
- Phase 6 (US3 - 호환성): 9 tasks
- Phase 7 (US5 - 목록 조회): 10 tasks
- Phase 8 (Polish): 12 tasks
