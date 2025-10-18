# Tasks: 프론트엔드 유저 세션 UI 통합

**Feature**: 004-frontend-user-session  
**Input**: Design documents from `/workspaces/realtime-svg/specs/004-frontend-user-session/`

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, etc.)
- Include exact file paths in descriptions

## Path Convention
- **Frontend**: `crates/frontend/src/`
- **Dependencies**: `crates/frontend/Cargo.toml`
- **Tests**: OPTIONAL - not included per spec (no test requirement mentioned)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and dependency configuration

- [X] T001 Add yew-router 0.18 dependency to crates/frontend/Cargo.toml
- [X] T002 [P] Add base64 0.22 dependency to crates/frontend/Cargo.toml
- [X] T003 [P] Add web-sys Storage feature to crates/frontend/Cargo.toml dependencies
- [X] T004 [P] Create module structure: auth/, components/, api/ directories in crates/frontend/src/
- [X] T005 Create types.rs for shared type definitions in crates/frontend/src/types.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T006 Define Route enum with yew-router in crates/frontend/src/routes.rs
- [ ] T007 [P] Implement Claims struct with from_token() and is_expired() in crates/frontend/src/auth/token.rs
- [ ] T008 [P] Implement TokenStorage trait and LocalTokenStorage in crates/frontend/src/auth/storage.rs
- [ ] T009 [P] Create AuthState enum (Anonymous, Authenticated) in crates/frontend/src/auth/mod.rs
- [ ] T010 [P] Create AuthContext struct with login/logout callbacks in crates/frontend/src/auth/mod.rs
- [ ] T011 Implement AuthProvider component in crates/frontend/src/auth/mod.rs
- [ ] T012 [P] Create ApiState<T> enum in crates/frontend/src/types.rs
- [ ] T013 [P] Implement handle_response() for 401/403/404 detection in crates/frontend/src/api/mod.rs
- [ ] T014 Implement AuthenticatedRequest helper for auto-adding Bearer token in crates/frontend/src/api/mod.rs
- [ ] T015 Update lib.rs to wrap app in AuthProvider and BrowserRouter in crates/frontend/src/lib.rs

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - JWT 토큰 발급 UI (Priority: P0) 🎯 MVP

**Goal**: 사용자가 user_id를 입력하여 JWT 토큰을 발급받고, localStorage에 저장하며, 로그인 상태를 복원할 수 있다.

**Independent Test**: user_id 입력 → 토큰 발급 → localStorage 확인 → 페이지 새로고침 → 로그인 상태 유지

### Implementation for User Story 1

- [ ] T016 [P] [US1] Create TokenResponse struct in crates/frontend/src/types.rs
- [ ] T017 [P] [US1] Implement request_token() function in crates/frontend/src/api/auth.rs
- [ ] T018 [US1] Create LoginForm component with user_id input in crates/frontend/src/components/login_form.rs
- [ ] T019 [US1] Add login form submission logic with spawn_local in crates/frontend/src/components/login_form.rs
- [ ] T020 [US1] Implement token storage on successful response in crates/frontend/src/components/login_form.rs
- [ ] T021 [US1] Add error handling for token request failures in crates/frontend/src/components/login_form.rs
- [ ] T022 [P] [US1] Create Header component showing login state in crates/frontend/src/components/header.rs
- [ ] T023 [US1] Display current user_id from AuthContext in Header component in crates/frontend/src/components/header.rs
- [ ] T024 [US1] Update Home route to show LoginForm when Anonymous in crates/frontend/src/lib.rs

**Checkpoint**: User Story 1 완료 - 사용자가 로그인하고 상태가 유지됨

---

## Phase 4: User Story 2 - 유저별 세션 생성 UI (Priority: P1)

**Goal**: 로그인한 사용자가 유저별 세션을 생성하고 상세 페이지로 이동할 수 있다.

**Independent Test**: 로그인 → 유저별 세션 생성 폼 입력 → 세션 생성 → `/session/{user_id}/{session_id}` 리다이렉트

### Implementation for User Story 2

- [ ] T025 [P] [US2] Create UserSessionCreateRequest struct in crates/frontend/src/types.rs
- [ ] T026 [P] [US2] Implement create_user_session() in crates/frontend/src/api/user_session.rs
- [ ] T027 [US2] Refactor existing CreatePage to SessionForm component in crates/frontend/src/components/session_form.rs
- [ ] T028 [US2] Add mode prop to SessionForm (UserMode vs PublicMode) in crates/frontend/src/components/session_form.rs
- [ ] T029 [US2] Implement user session creation logic with AuthContext check in crates/frontend/src/components/session_form.rs
- [ ] T030 [US2] Extract user_id from AuthContext Claims in crates/frontend/src/components/session_form.rs
- [ ] T031 [US2] Handle "로그인이 필요합니다" error when Anonymous in crates/frontend/src/components/session_form.rs
- [ ] T032 [US2] Add 409 Conflict error handling for duplicate session_id in crates/frontend/src/components/session_form.rs
- [ ] T033 [US2] Implement redirect to /session/{user_id}/{session_id} on success in crates/frontend/src/components/session_form.rs
- [ ] T034 [US2] Update Home route to show SessionForm with mode selection in crates/frontend/src/lib.rs

**Checkpoint**: User Story 2 완료 - 유저별 세션 생성 가능

---

## Phase 5: User Story 3 - 유저별 세션 수정 UI (Priority: P1)

**Goal**: 사용자가 자신의 세션 파라미터를 수정할 수 있으며, 다른 사용자는 수정이 차단된다.

**Independent Test**: 로그인 → 자신의 세션 상세 페이지 → args 수정 → 업데이트 성공 / 다른 사용자로 시도 → 403 에러

### Implementation for User Story 3

- [ ] T035 [P] [US3] Create SessionDetail, SessionUpdateRequest structs in crates/frontend/src/types.rs
- [ ] T036 [P] [US3] Implement get_user_session_detail() in crates/frontend/src/api/user_session.rs
- [ ] T037 [P] [US3] Implement update_user_session() in crates/frontend/src/api/user_session.rs
- [ ] T038 [US3] Refactor existing DetailPage to SessionDetailPage component in crates/frontend/src/components/session_detail.rs
- [ ] T039 [US3] Add is_user_session prop to SessionDetailPage in crates/frontend/src/components/session_detail.rs
- [ ] T040 [US3] Implement session detail fetch with route params in crates/frontend/src/components/session_detail.rs
- [ ] T041 [US3] Add args textarea editor with JSON validation in crates/frontend/src/components/session_detail.rs
- [ ] T042 [US3] Implement update button with AuthContext check in crates/frontend/src/components/session_detail.rs
- [ ] T043 [US3] Handle 403 Forbidden error with "권한이 없습니다" message in crates/frontend/src/components/session_detail.rs
- [ ] T044 [US3] Handle 401 Unauthorized with auto-logout logic in crates/frontend/src/components/session_detail.rs
- [ ] T045 [US3] Add stream preview refresh on successful update in crates/frontend/src/components/session_detail.rs
- [ ] T046 [US3] Update stream URL to /stream/{user_id}/{session_id} for user sessions in crates/frontend/src/components/session_detail.rs
- [ ] T047 [US3] Add UserSession route handling in switch function in crates/frontend/src/lib.rs

**Checkpoint**: User Story 3 완료 - 유저별 세션 수정 및 권한 검증 작동

---

## Phase 6: User Story 4 - 유저별 세션 목록 조회 UI (Priority: P2)

**Goal**: 사용자가 자신이 생성한 모든 세션 목록을 조회하고 상세 페이지로 이동할 수 있다.

**Independent Test**: 로그인 → 여러 세션 생성 → "내 세션 목록" 클릭 → 모든 세션 표시 → 세션 클릭 → 상세 페이지 이동

### Implementation for User Story 4

- [ ] T048 [P] [US4] Create SessionListItem, SessionListResponse structs in crates/frontend/src/types.rs
- [ ] T049 [P] [US4] Implement list_user_sessions() in crates/frontend/src/api/user_session.rs
- [ ] T050 [US4] Create SessionListPage component in crates/frontend/src/components/session_list.rs
- [ ] T051 [US4] Fetch sessions on mount with AuthContext user_id in crates/frontend/src/components/session_list.rs
- [ ] T052 [US4] Render session list as cards with session_id, created_at, ttl in crates/frontend/src/components/session_list.rs
- [ ] T053 [US4] Handle empty list with "생성된 세션이 없습니다" message in crates/frontend/src/components/session_list.rs
- [ ] T054 [US4] Add "새 세션 만들기" button when list is empty in crates/frontend/src/components/session_list.rs
- [ ] T055 [US4] Implement session card click navigation to /session/{user_id}/{session_id} in crates/frontend/src/components/session_list.rs
- [ ] T056 [US4] Add MySessions route handling in switch function in crates/frontend/src/lib.rs
- [ ] T057 [US4] Add "내 세션 목록" link in Header component in crates/frontend/src/components/header.rs

**Checkpoint**: User Story 4 완료 - 세션 목록 조회 가능

---

## Phase 7: User Story 5 - 공용 세션 호환성 유지 (Priority: P1)

**Goal**: 기존 공용 세션 생성/수정 기능이 계속 작동하며, 유저별 세션과 공존할 수 있다.

**Independent Test**: 비로그인 상태 → 공용 세션 생성 → `/session/{session_id}` 접근 → 수정 성공

### Implementation for User Story 5

- [ ] T058 [P] [US5] Create PublicSessionCreateRequest struct in crates/frontend/src/types.rs
- [ ] T059 [P] [US5] Implement create_public_session() in crates/frontend/src/api/public_session.rs
- [ ] T060 [P] [US5] Implement update_public_session() in crates/frontend/src/api/public_session.rs
- [ ] T061 [P] [US5] Implement get_public_session_detail() in crates/frontend/src/api/public_session.rs
- [ ] T062 [US5] Add PublicMode handling to SessionForm component in crates/frontend/src/components/session_form.rs
- [ ] T063 [US5] Implement public session creation without auth check in crates/frontend/src/components/session_form.rs
- [ ] T064 [US5] Add redirect to /session/{session_id} on public session creation in crates/frontend/src/components/session_form.rs
- [ ] T065 [US5] Add PublicSession route handling in switch function in crates/frontend/src/lib.rs
- [ ] T066 [US5] Update SessionDetailPage to handle public sessions (no auth) in crates/frontend/src/components/session_detail.rs
- [ ] T067 [US5] Update stream URL to /stream/{session_id} for public sessions in crates/frontend/src/components/session_detail.rs
- [ ] T068 [US5] Ensure update button works without token for public sessions in crates/frontend/src/components/session_detail.rs

**Checkpoint**: User Story 5 완료 - 공용 세션 하위 호환성 보장

---

## Phase 8: User Story 6 - 로그아웃 및 토큰 관리 (Priority: P2)

**Goal**: 사용자가 로그아웃하여 토큰을 삭제하고 비로그인 상태로 전환할 수 있다.

**Independent Test**: 로그인 → "로그아웃" 클릭 → localStorage 토큰 삭제 확인 → 비로그인 상태로 전환

### Implementation for User Story 6

- [ ] T069 [US6] Add "로그아웃" button to Header component (visible when Authenticated) in crates/frontend/src/components/header.rs
- [ ] T070 [US6] Implement logout callback handler in Header component in crates/frontend/src/components/header.rs
- [ ] T071 [US6] Trigger AuthContext.logout() on button click in crates/frontend/src/components/header.rs
- [ ] T072 [US6] Add redirect to home (/) after logout in crates/frontend/src/components/header.rs
- [ ] T073 [US6] Update UI to hide "내 세션 목록" link when Anonymous in crates/frontend/src/components/header.rs
- [ ] T074 [US6] Update SessionForm to disable user mode when Anonymous in crates/frontend/src/components/session_form.rs

**Checkpoint**: User Story 6 완료 - 로그아웃 기능 작동

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T075 [P] Add wasm-logger initialization in crates/frontend/src/lib.rs
- [ ] T076 [P] Add log::info for successful operations across all API calls
- [ ] T077 [P] Add log::warn for 401/403 responses in handle_response()
- [ ] T078 [P] Add log::error for network failures and parsing errors
- [ ] T079 [P] Update NotFound route with 404 page component in crates/frontend/src/lib.rs
- [ ] T080 [P] Add CSS styling for error messages (.error, .success, .loading) in crates/frontend/styles.css
- [ ] T081 [P] Add loading spinners for ApiState::Loading states
- [ ] T082 Code review: Ensure all components use AuthContext correctly
- [ ] T083 Code review: Verify all API calls have proper error handling
- [ ] T084 Manual testing: Run all quickstart.md scenarios
- [ ] T085 [P] Update index.html title and meta tags if needed in crates/frontend/index.html

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-8)**: All depend on Foundational phase completion
  - US1 (P0): No dependencies on other stories - MUST complete first (MVP blocker)
  - US2 (P1): Depends on US1 (needs AuthContext) - Can start after US1
  - US3 (P1): Depends on US2 (needs session creation) - Can start after US2
  - US4 (P2): Depends on US2 (needs sessions to list) - Can start after US2
  - US5 (P1): Independent - Can run in parallel with US2/US3/US4
  - US6 (P2): Depends on US1 (needs AuthContext) - Can run in parallel with US4/US5
- **Polish (Phase 9)**: Depends on all desired user stories being complete

### User Story Dependencies

- **US1 (P0)**: Foundation → US1 (MVP - CRITICAL)
- **US2 (P1)**: Foundation + US1 → US2
- **US3 (P1)**: Foundation + US2 → US3
- **US4 (P2)**: Foundation + US2 → US4 (parallel with US3)
- **US5 (P1)**: Foundation only → US5 (parallel with US2/US3/US4)
- **US6 (P2)**: Foundation + US1 → US6 (parallel with US4/US5)

### Within Each User Story

- API functions before UI components
- Type definitions before functions that use them
- Core implementation before error handling
- Story complete before moving to next priority

### Parallel Opportunities

**Setup Phase (T001-T005)**:
- T002, T003 can run in parallel (different Cargo.toml sections)
- T004, T005 can run in parallel (different files)

**Foundational Phase (T006-T015)**:
- T007, T008, T009, T010, T012, T013, T014 can all run in parallel (different files)

**User Story 1 (T016-T024)**:
- T016, T017 can run in parallel (different files)
- T022, T023 can run in parallel after T021

**User Story 2 (T025-T034)**:
- T025, T026 can run in parallel

**User Story 3 (T035-T047)**:
- T035, T036, T037 can all run in parallel

**User Story 4 (T048-T057)**:
- T048, T049 can run in parallel

**User Story 5 (T058-T068)**:
- T058, T059, T060, T061 can all run in parallel

**Polish Phase (T075-T085)**:
- T075, T076, T077, T078, T079, T080, T081 can all run in parallel

---

## Parallel Example: User Story 1 (MVP)

```bash
# After Foundational phase completes, launch in parallel:
Task T016: "Create TokenResponse struct in crates/frontend/src/types.rs"
Task T017: "Implement request_token() in crates/frontend/src/api/auth.rs"

# After T020 completes, launch in parallel:
Task T022: "Create Header component in crates/frontend/src/components/header.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only = P0)

1. Complete Phase 1: Setup (T001-T005)
2. Complete Phase 2: Foundational (T006-T015) - CRITICAL BLOCKER
3. Complete Phase 3: User Story 1 (T016-T024)
4. **STOP and VALIDATE**: Test JWT 토큰 발급, localStorage 저장, 페이지 새로고침 시 상태 복원
5. Deploy/demo if ready - **This is MVP**

### Incremental Delivery (Recommended Order)

1. Setup + Foundational → Foundation ready
2. **Add US1 (P0)** → Test independently → **Deploy (MVP!)** ✅
3. Add US2 (P1) → Test independently → Deploy
4. Add US3 (P1) → Test independently → Deploy
5. Add US5 (P1) → Test independently → Deploy (backward compatibility)
6. Add US4 (P2) → Test independently → Deploy (UX enhancement)
7. Add US6 (P2) → Test independently → Deploy (security enhancement)
8. Polish → Final release

### Parallel Team Strategy

With 2 developers after Foundational phase:

1. **Week 1**:
   - Dev A: US1 (P0) - CRITICAL, must complete first
   
2. **Week 2** (after US1 completes):
   - Dev A: US2 (P1)
   - Dev B: US5 (P1) - Independent, backward compatibility
   
3. **Week 3**:
   - Dev A: US3 (P1) - Depends on US2
   - Dev B: US4 (P2) - Depends on US2, can start after US2
   
4. **Week 4**:
   - Dev A: US6 (P2)
   - Dev B: Polish (Phase 9)

---

## Notes

- [P] tasks = different files, no dependencies, safe to parallelize
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
- **MVP = US1 only** - Everything else is incremental enhancement
- **US5 (공용 세션 호환성) is P1** - Critical for backward compatibility, prioritize early

## Total Task Count

- **Setup**: 5 tasks
- **Foundational**: 10 tasks
- **User Story 1 (P0)**: 9 tasks
- **User Story 2 (P1)**: 10 tasks
- **User Story 3 (P1)**: 13 tasks
- **User Story 4 (P2)**: 10 tasks
- **User Story 5 (P1)**: 11 tasks
- **User Story 6 (P2)**: 6 tasks
- **Polish**: 11 tasks

**Total**: 85 tasks

**Parallel Opportunities**: 20+ tasks can run in parallel within their phases
