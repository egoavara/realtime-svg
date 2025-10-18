# Tasks: CLI Configuration Enhancement

**Input**: Design documents from `/specs/005-cli-config-enhancement/`  
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md

**Tests**: Constitution requires testing discipline (Principle IV). Tests are included based on data-model.md test scenarios.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions
- **Rust workspace**: `crates/common/src/`, `crates/backend/src/`, `tests/` at repository root
- Paths follow plan.md structure: common crate for config, backend for main.rs integration

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add dependencies and basic project configuration

- [x] T001 Add clap, figment, dotenvy to workspace Cargo.toml dependencies
- [x] T002 [P] Add figment YAML feature to workspace Cargo.toml
- [x] T003 [P] Update crates/common/Cargo.toml to include clap, figment, dotenvy, anyhow

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core Config infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 Create crates/common/src/config.rs with Config struct (redis_url, host, port, log_level fields)
- [x] T005 [P] Add CliArgs struct in crates/common/src/config.rs using clap derive API
- [x] T006 Implement Config::default() with default values from data-model.md
- [x] T007 Implement custom Debug trait for Config in crates/common/src/config.rs (mask redis_url per research.md #6)
- [x] T008 Add pub mod config to crates/common/src/lib.rs

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - 기본 설정 파일로 서버 실행 (Priority: P1) 🎯 MVP

**Goal**: 운영자가 별도 인자 없이 서버를 실행하면 config.yaml을 자동으로 읽어 서버가 시작된다

**Independent Test**: 바이너리 실행 디렉토리에 config.yaml을 배치한 후 인자 없이 실행하여 설정이 적용되는지 확인

### Tests for User Story 1

**NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T009 [P] [US1] Create tests/config_test.rs with test_default_config (기본값 테스트)
- [x] T010 [P] [US1] Add test_yaml_file_loading in tests/config_test.rs (YAML 파일 로딩 테스트)
- [x] T011 [P] [US1] Add test_yaml_parsing_error in tests/config_test.rs (잘못된 YAML 파일 파싱 오류 테스트)

### Implementation for User Story 1

- [x] T012 [US1] Implement resolve_config_path helper function in crates/common/src/config.rs (per research.md #4)
- [x] T013 [US1] Implement Config::load() with Figment YAML layer in crates/common/src/config.rs
- [x] T014 [US1] Add Config::validate() method in crates/common/src/config.rs (port range, redis_url prefix validation per research.md #5)
- [x] T015 [US1] Create config.yaml.example at repository root with example configuration
- [x] T016 [US1] Update crates/backend/src/main.rs to call Config::load() and validate()
- [x] T017 [US1] Add tracing::info! log for final config in crates/backend/src/main.rs (FR-012)
- [x] T018 [US1] Replace std::env::var("REDIS_URL") with config.redis_url in crates/backend/src/main.rs
- [x] T019 [US1] Update server bind address to use config.host and config.port in crates/backend/src/main.rs

**Checkpoint**: US1 완료 - config.yaml 파일로 서버 실행 가능, 검증 로직 동작, 기본값 fallback 지원

---

## Phase 4: User Story 2 - 환경 변수로 설정 오버라이드 (Priority: P1)

**Goal**: 운영자가 환경 변수를 설정하면 설정 파일보다 환경 변수 값이 우선 적용된다

**Independent Test**: config.yaml과 환경 변수를 모두 설정한 후 서버를 실행하여 환경 변수 값이 우선 적용되는지 확인

### Tests for User Story 2

- [x] T020 [P] [US2] Add test_env_overrides_yaml in tests/config_test.rs (ENV > Config 우선순위 테스트)
- [x] T021 [P] [US2] Add test_dotenv_loading in tests/config_test.rs (.env 파일 로딩 테스트)

### Implementation for User Story 2

- [x] T022 [US2] Add dotenvy::dotenv().ok() call at start of crates/backend/src/main.rs (per research.md #3)
- [x] T023 [US2] Add Env::prefixed layer to Figment in Config::load() in crates/common/src/config.rs (no prefix for this project)
- [x] T024 [US2] Update Figment merge order: YAML → ENV in crates/common/src/config.rs
- [x] T025 [US2] Create .env.example at repository root with example environment variables
- [x] T026 [US2] Update CliArgs with #[arg(env = "...")] attributes for all fields in crates/common/src/config.rs (per research.md #2)

**Checkpoint**: US2 완료 - 환경 변수로 config.yaml 오버라이드 가능, .env 파일 지원

---

## Phase 5: User Story 3 - 커맨드라인 옵션으로 최우선 설정 (Priority: P1)

**Goal**: 운영자가 커맨드라인 옵션으로 값을 전달하면 환경 변수와 설정 파일보다 CLI 옵션이 최우선 적용

**Independent Test**: 설정 파일, 환경 변수, 커맨드라인 옵션을 모두 다른 값으로 설정 후 CLI 옵션이 최우선 적용되는지 확인

### Tests for User Story 3

- [x] T027 [P] [US3] Add test_cli_overrides_all in tests/config_test.rs (CLI > ENV > Config 우선순위 전체 테스트)
- [x] T028 [P] [US3] Add test_priority_same_field in tests/config_test.rs (동일 필드 세 소스 설정 시 우선순위 테스트)

### Implementation for User Story 3

- [x] T029 [US3] Implement CliArgs parsing with clap::Parser::parse() in Config::load() in crates/common/src/config.rs
- [x] T030 [US3] Add Serialized::defaults(cli_args) layer to Figment in Config::load() in crates/common/src/config.rs
- [x] T031 [US3] Update Figment merge order: YAML → ENV → CLI in crates/common/src/config.rs (final order per FR-005)
- [x] T032 [US3] Add clap help text with env variable names in CliArgs derives in crates/common/src/config.rs

**Checkpoint**: US3 완료 - CLI > ENV > Config 우선순위 완전 구현, --help 옵션 지원

---

## Phase 6: User Story 4 - 커스텀 설정 파일 경로 지정 (Priority: P2)

**Goal**: 운영자가 --config 옵션으로 설정 파일 경로를 지정하면 지정된 경로의 설정 파일이 로드

**Independent Test**: 임의 경로에 설정 파일 생성 후 --config 옵션으로 지정하여 실행 시 설정 적용 확인

### Tests for User Story 4

- [x] T033 [P] [US4] Add test_custom_config_path_absolute in tests/config_test.rs (절대 경로 설정 파일 테스트)
- [x] T034 [P] [US4] Add test_custom_config_path_relative in tests/config_test.rs (상대 경로 바이너리 디렉토리 기준 해석 테스트)
- [x] T035 [P] [US4] Add test_config_file_not_found_error in tests/config_test.rs (존재하지 않는 경로 오류 테스트)

### Implementation for User Story 4

- [x] T036 [US4] Update resolve_config_path to handle both absolute and relative paths in crates/common/src/config.rs (per research.md #4)
- [x] T037 [US4] Update Config::load() to use config path from CliArgs in crates/common/src/config.rs
- [x] T038 [US4] Add error context for file not found in Config::load() in crates/common/src/config.rs (FR-009)

**Checkpoint**: US4 완료 - --config 옵션으로 커스텀 경로 지정 가능, 절대/상대 경로 지원

---

## Phase 7: User Story 5 - 설정 검증 및 도움말 (Priority: P3)

**Goal**: --help 옵션으로 모든 설정 항목 표시, 설정 검증 자동 수행

**Independent Test**: --help 옵션으로 도움말 확인, 잘못된 설정으로 실행 시 검증 로직 동작 확인

### Tests for User Story 5

- [x] T039 [P] [US5] Add test_validation_port_range in tests/config_test.rs (포트 범위 검증 테스트)
- [x] T040 [P] [US5] Add test_validation_redis_url_prefix in tests/config_test.rs (Redis URL 프로토콜 검증 테스트)
- [x] T041 [P] [US5] Add test_validation_error_messages in tests/config_test.rs (검증 오류 메시지 명확성 테스트)

### Implementation for User Story 5

- [x] T042 [US5] Enhance Config::validate() with comprehensive validation rules in crates/common/src/config.rs (per data-model.md "Validation Rules")
- [x] T043 [US5] Add host field validation (non-empty) in Config::validate() in crates/common/src/config.rs
- [x] T044 [US5] Add log_level field validation (tracing::EnvFilter compatible) in Config::validate() in crates/common/src/config.rs
- [x] T045 [US5] Add detailed clap help messages for each field in CliArgs in crates/common/src/config.rs (FR-010)
- [x] T046 [US5] Document environment variable names in clap help text in crates/common/src/config.rs

**Checkpoint**: US5 완료 - 모든 설정 항목 검증, --help 도움말 완비

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T047 [P] Add unit test for Debug trait masking in tests/config_test.rs
- [x] T048 [P] Add integration test for full config lifecycle in tests/config_test.rs
- [x] T049 Run cargo test --workspace to verify all tests pass
- [x] T050 Run cargo clippy --workspace -- -D warnings to ensure no warnings
- [x] T051 Run cargo fmt --check to verify formatting
- [ ] T052 [P] Update README.md with configuration usage examples (reference quickstart.md)
- [ ] T053 Validate quickstart.md scenarios manually (기본 실행, .env, config.yaml, --config, --help)
- [x] T054 Remove hardcoded HOST/PORT defaults from backend if any, ensure only config.rs has defaults
- [x] T055 Add inline documentation comments to Config and CliArgs structs in crates/common/src/config.rs

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - User stories can proceed sequentially in priority order (P1 → P1 → P1 → P2 → P3)
  - Or P1 stories (US1, US2, US3) can proceed in parallel after Foundational
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P1)**: Depends on User Story 1 completion (builds on Config::load() implementation)
- **User Story 3 (P1)**: Depends on User Story 2 completion (adds CLI layer to existing YAML+ENV layers)
- **User Story 4 (P2)**: Depends on User Story 3 completion (enhances Config::load() with custom path logic)
- **User Story 5 (P3)**: Depends on all P1 and P2 stories (enhances validation and documentation)

**Rationale for Sequential Dependencies**: Each user story extends the configuration system incrementally:
- US1: YAML file loading
- US2: + ENV layer (requires Figment merge from US1)
- US3: + CLI layer (requires ENV merge from US2)
- US4: + Custom path (requires CLI parsing from US3)
- US5: + Validation & Help (requires full config stack from US1-4)

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Helper functions (resolve_config_path) before main logic (Config::load)
- Core implementation (Config::load, validate) before integration (main.rs changes)
- Story complete and tested before moving to next priority

### Parallel Opportunities

- All Setup tasks (T001-T003) can run in parallel
- All Foundational tasks marked [P] (T005) can run in parallel with T004
- Within each story, tests marked [P] can run in parallel
- US5 validation tests (T039-T041) can run in parallel
- Polish tasks marked [P] (T047, T048, T052) can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Create tests/config_test.rs with test_default_config"
Task: "Add test_yaml_file_loading in tests/config_test.rs"
Task: "Add test_yaml_parsing_error in tests/config_test.rs"

# After tests fail, implement sequentially:
# T012 → T013 → T014 → T015 → T016 → T017 → T018 → T019
```

---

## Parallel Example: User Story 5

```bash
# Launch all validation tests together:
Task: "Add test_validation_port_range in tests/config_test.rs"
Task: "Add test_validation_redis_url_prefix in tests/config_test.rs"
Task: "Add test_validation_error_messages in tests/config_test.rs"

# After tests fail, implement validations:
# T042 → T043 → T044 → T045 → T046 (sequential, same file)
```

---

## Implementation Strategy

### MVP First (User Stories 1-3 - All P1)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational (T004-T008) - **CRITICAL**
3. Complete Phase 3: User Story 1 (T009-T019)
4. **CHECKPOINT**: Test US1 independently with config.yaml
5. Complete Phase 4: User Story 2 (T020-T026)
6. **CHECKPOINT**: Test US2 independently with ENV override
7. Complete Phase 5: User Story 3 (T027-T032)
8. **CHECKPOINT**: Test US3 independently with CLI override
9. **MVP READY**: All P1 stories complete - full config system functional

### Incremental Delivery

1. Setup + Foundational → Foundation ready (T001-T008)
2. Add User Story 1 → Test independently → Deploy/Demo (config.yaml support)
3. Add User Story 2 → Test independently → Deploy/Demo (+ ENV support)
4. Add User Story 3 → Test independently → Deploy/Demo (+ CLI support) **← MVP**
5. Add User Story 4 → Test independently → Deploy/Demo (+ Custom path)
6. Add User Story 5 → Test independently → Deploy/Demo (+ Validation/Help)
7. Polish → Final release (T047-T055)

### Single Developer Strategy

Follow user stories sequentially in priority order:

1. Complete Setup + Foundational (T001-T008)
2. Complete US1: Basic config.yaml (T009-T019)
3. Complete US2: ENV override (T020-T026)
4. Complete US3: CLI override (T027-T032) **← Stop here for MVP**
5. Complete US4: Custom path (T033-T038)
6. Complete US5: Validation (T039-T046)
7. Complete Polish (T047-T055)

**Recommended MVP Scope**: Phases 1-5 (through User Story 3) provides full CLI > ENV > Config hierarchy

---

## Notes

- [P] tasks = different files or independent test cases, no dependencies
- [Story] label maps task to specific user story for traceability (US1-US5)
- Each user story extends the previous (US1 → US2 → US3 → US4 → US5)
- Verify tests fail before implementing (TDD)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Constitution compliance: All tests included per Principle IV (Testing Discipline)
- Breaking changes: None (환경 변수 REDIS_URL remains supported per SC-006)

## Summary

- **Total Tasks**: 55
- **User Story 1**: 11 tasks (3 tests + 8 implementation)
- **User Story 2**: 7 tasks (2 tests + 5 implementation)
- **User Story 3**: 6 tasks (2 tests + 4 implementation)
- **User Story 4**: 6 tasks (3 tests + 3 implementation)
- **User Story 5**: 8 tasks (3 tests + 5 implementation)
- **Setup/Foundational**: 8 tasks
- **Polish**: 9 tasks
- **Parallel Opportunities**: 19 tasks marked [P]
- **MVP Scope**: Phases 1-5 (35 tasks, US1-US3 complete)
- **Independent Test Criteria**: Each user story has checkpoint with specific test scenario from spec.md
