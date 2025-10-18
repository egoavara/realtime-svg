# Tasks: 멀티플랫폼 Docker 빌드 설정

**Input**: Design documents from `/specs/006-multiplatform-docker-build/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Verification tasks are included for each user story's acceptance criteria

**Organization**: Tasks are grouped by user story to enable independent implementation and testing

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions
- Files at repository root: `Dockerfile`, `.dockerignore`
- GitHub Actions: `.github/workflows/docker-build.yml`
- Existing workspace structure unchanged

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Docker build infrastructure preparation and prerequisites

- [ ] T001 Verify Docker Buildx is installed and create multiplatform builder instance (Manual - requires Docker on host)
- [X] T002 [P] Create .dockerignore file at repository root with target/, .git/, specs/ exclusions
- [ ] T003 [P] Document multiplatform build commands in README.md (optional reference section)

**Checkpoint**: Build infrastructure ready

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core Dockerfile structure that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 Create base Dockerfile at repository root with ARG TARGETPLATFORM declaration
- [X] T005 [P] Add platform-to-Rust-target mapping logic (case statement for amd64, arm64, armv7)
- [X] T006 [P] Setup cross-compiler installation commands (gcc-aarch64-linux-gnu, gcc-arm-linux-gnueabihf, etc.)
- [X] T007 Configure Rust target addition based on TARGETPLATFORM variable

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - 멀티플랫폼 이미지 빌드 (Priority: P1) 🎯 MVP

**Goal**: 개발자가 단일 명령으로 AMD64, ARM64, ARMv7 플랫폼용 Docker 이미지를 빌드하고 레지스트리에 푸시할 수 있다

**Independent Test**: 
```bash
docker buildx build --platform linux/amd64,linux/arm64,linux/arm/v7 \
  --tag test-image:latest --load .
docker manifest inspect test-image:latest
```
출력에 3개 플랫폼이 포함되는지 확인

### Implementation for User Story 1

- [X] T008 [P] [US1] Add Chef stage to Dockerfile using lukemathwalker/cargo-chef:latest-rust-1.86 base image
- [X] T009 [P] [US1] Add Planner stage to Dockerfile for cargo chef prepare with recipe.json output
- [X] T010 [US1] Add Builder stage with --platform=$BUILDPLATFORM flag in FROM instruction
- [X] T011 [US1] Implement rustup target add command using $RUST_TARGET from mapping
- [X] T012 [US1] Add error handling for unsupported TARGETPLATFORM values (exit 1 with clear message)
- [X] T013 [US1] Add WORKDIR /app and COPY commands for source code in Builder stage
- [ ] T014 [US1] Test local single-platform build with docker build command for current architecture (Manual - requires Docker)

**Checkpoint**: User Story 1 완료 - 로컬에서 단일 플랫폼 빌드 성공

---

## Phase 4: User Story 2 - Rust 크로스 컴파일 최적화 (Priority: P1)

**Goal**: Dockerfile이 각 플랫폼별 타겟으로 크로스 컴파일하여 올바른 바이너리를 생성한다

**Independent Test**:
```bash
docker buildx build --platform linux/arm64 --tag test-backend:arm64 --load .
docker run --rm --entrypoint file test-backend:arm64 /usr/local/bin/backend
```
출력: `ELF 64-bit LSB executable, ARM aarch64` 확인

### Implementation for User Story 2

- [X] T015 [P] [US2] Add linker environment variable configuration for aarch64 (CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc)
- [X] T016 [P] [US2] Add linker environment variable configuration for armv7 (CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc)
- [X] T017 [US2] Implement cargo build command with --target $RUST_TARGET --bin backend flags
- [X] T018 [US2] Add mv command to relocate binary from target/$RUST_TARGET/release/backend to /app/backend
- [ ] T019 [US2] Test cross-compilation for ARM64 platform and verify binary architecture with file command (Manual - requires Docker)
- [ ] T020 [US2] Test cross-compilation for ARMv7 platform and verify binary architecture with file command (Manual - requires Docker)

**Checkpoint**: User Story 2 완료 - 크로스 컴파일로 올바른 아키텍처 바이너리 생성

---

## Phase 5: User Story 3 - 빌드 캐싱 최적화 (Priority: P2)

**Goal**: cargo-chef를 사용하여 종속성 빌드 캐싱으로 빌드 시간을 50% 단축한다

**Independent Test**:
```bash
# 첫 빌드 시간 측정
time docker buildx build --platform linux/amd64 --tag test:v1 --load .
# 소스 코드만 수정 (Cargo.toml 변경 없이)
echo "// comment" >> crates/backend/src/main.rs
# 재빌드 시간 측정 (50% 이하여야 함)
time docker buildx build --platform linux/amd64 --tag test:v2 --load .
```

### Implementation for User Story 3

- [X] T021 [P] [US3] Modify Planner stage to COPY entire workspace (COPY . .) for recipe generation
- [X] T022 [US3] Add Cook stage after Planner to run cargo chef cook with recipe.json
- [X] T023 [US3] Configure Cook stage with --release and --target $RUST_TARGET flags
- [X] T024 [US3] Separate source code COPY command AFTER cargo chef cook to preserve cache layer
- [ ] T025 [US3] Test cache efficiency by modifying src/ files only and verifying cook stage is cached (Manual - requires Docker)
- [ ] T026 [US3] Test cache invalidation by adding new crate to Cargo.toml and verifying rebuild (Manual - requires Docker)

**Checkpoint**: User Story 3 완료 - 종속성 캐싱으로 빌드 시간 50% 이상 단축

---

## Phase 6: User Story 4 - 최종 이미지 크기 최소화 (Priority: P2)

**Goal**: 멀티 스테이지 빌드로 최종 이미지 크기를 100MB 이하로 유지한다

**Independent Test**:
```bash
docker buildx build --platform linux/amd64 --tag test-backend:slim --load .
docker images test-backend:slim
# SIZE 열이 100MB 이하인지 확인
```

### Implementation for User Story 4

- [X] T027 [P] [US4] Add Runtime stage with FROM debian:bookworm-slim AS runtime
- [X] T028 [P] [US4] Install runtime dependencies (ca-certificates, tini) with apt-get in Runtime stage
- [X] T029 [US4] Add COPY --from=builder command to copy binary from /app/backend to /usr/local/bin/backend
- [X] T030 [US4] Configure ENTRYPOINT to use tini wrapper: ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/backend"]
- [X] T031 [US4] Add chmod +x command for binary executable permissions in Runtime stage
- [ ] T032 [US4] Test final image size with docker images command and verify <100MB (Manual - requires Docker)
- [ ] T033 [US4] Test runtime execution with docker run --rm test-backend:slim --help command (Manual - requires Docker)

**Checkpoint**: User Story 4 완료 - 최종 이미지 크기 100MB 이하 달성

---

## Phase 7: User Story 5 - GitHub Actions CI/CD 통합 (Priority: P3)

**Goal**: GitHub Actions 워크플로우에서 멀티플랫폼 이미지를 자동으로 빌드하고 Docker Hub에 푸시한다

**Independent Test**:
1. PR 생성 → GitHub Actions에서 빌드 성공 (푸시 안 됨) 확인
2. main 병합 → Docker Hub에 이미지 푸시 확인
3. `docker pull username/realtime-svg-backend:latest` 성공 확인

### Implementation for User Story 5

- [X] T034 [P] [US5] Create .github/workflows directory if not exists
- [X] T035 [US5] Create docker-build.yml workflow file with name "Docker Multiplatform Build"
- [X] T036 [P] [US5] Add workflow trigger on push to main and pull_request events
- [X] T037 [P] [US5] Add Checkout step using actions/checkout@v4
- [X] T038 [P] [US5] Add QEMU setup step using docker/setup-qemu-action@v3
- [X] T039 [P] [US5] Add Docker Buildx setup step using docker/setup-buildx-action@v3
- [X] T040 [US5] Add Docker Hub login step using docker/login-action@v3 with secrets.DOCKERHUB_USERNAME and secrets.DOCKERHUB_TOKEN
- [X] T041 [US5] Add build and push step using docker/build-push-action@v6 with platforms: linux/amd64,linux/arm64,linux/arm/v7
- [X] T042 [US5] Configure conditional push logic: push only if github.event_name == 'push' AND github.ref == 'refs/heads/main'
- [X] T043 [US5] Add image tags: latest and github.sha in build-push-action step
- [X] T044 [US5] Configure cache-from and cache-to with type=gha,mode=max for GitHub Actions cache
- [X] T045 [US5] Add build-args for TARGET_BINARY=backend and RUST_VERSION=1.86
- [ ] T046 [US5] Document required GitHub Secrets (DOCKERHUB_USERNAME, DOCKERHUB_TOKEN) in workflow comments or README (Optional - can be added later)
- [ ] T047 [US5] Test workflow by creating PR and verifying build-only (no push) (Manual - requires PR creation)
- [ ] T048 [US5] Test workflow by merging to main and verifying Docker Hub image push (Manual - requires merge to main)

**Checkpoint**: User Story 5 완료 - CI/CD 파이프라인 자동화 완료

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Final refinements, documentation, and optional enhancements

- [X] T049 [P] Add Dockerfile comments explaining each stage purpose and ARG usage
- [X] T050 [P] Add .dockerignore comments for excluded file categories
- [ ] T051 [P] Create Dockerfile.frontend for frontend binary (optional, same structure as backend)
- [ ] T052 Update README.md with multiplatform build quickstart section (optional)
- [X] T053 Add Docker image labels (org.opencontainers.image.title, version, platform) in Runtime stage
- [ ] T054 Test complete multiplatform build pipeline end-to-end on clean environment (Manual - requires Docker)

**Checkpoint**: 모든 작업 완료 - 프로덕션 준비 완료

---

## Dependencies & Execution Order

### User Story Dependencies

```
Setup (T001-T003)
  ↓
Foundational (T004-T007) ← BLOCKING for all user stories
  ↓
  ├─→ US1 (T008-T014) ────→ MVP Complete
  ├─→ US2 (T015-T020) ────→ Can run after US1
  ├─→ US3 (T021-T026) ────→ Can run after US1
  ├─→ US4 (T027-T033) ────→ Can run after US1
  └─→ US5 (T034-T048) ────→ Can run after US1 + US2 + US3 + US4 (requires complete Dockerfile)
       ↓
    Polish (T049-T054)
```

**Key Insights**:
- **MVP**: Phase 1 + Phase 2 + Phase 3 (US1) = 로컬 단일 플랫폼 빌드 가능
- **US2-US4 are independent**: 크로스 컴파일(US2), 캐싱(US3), 이미지 최소화(US4)는 병렬 작업 가능
- **US5 requires all**: GitHub Actions는 완전한 Dockerfile 필요 (US1-US4 완료 후)

---

## Parallel Execution Examples

### Phase 2 Foundational Tasks (병렬 가능)
```bash
# 동시에 작업 가능한 태스크
[Developer A] T005 - Platform mapping logic
[Developer B] T006 - Cross-compiler installation
[Developer C] T007 - Rust target configuration
```

### Phase 3 US1 Tasks (병렬 가능)
```bash
[Developer A] T008 - Chef stage
[Developer B] T009 - Planner stage
# T010-T014는 순차 (같은 Dockerfile 수정)
```

### Phase 5-7 User Stories (독립적 병렬 가능)
```bash
[Team A] US3 (T021-T026) - 캐싱 최적화
[Team B] US4 (T027-T033) - 이미지 최소화
# 각 팀이 독립적으로 자신의 Dockerfile 버전 작업 후 병합
```

---

## Implementation Strategy

### Recommended Approach: MVP First

1. **Week 1: MVP (US1 + US2)**
   - Complete T001-T020
   - Deliverable: 로컬에서 멀티플랫폼 빌드 성공, 크로스 컴파일 검증 완료
   - Value: 개발자가 수동으로 멀티플랫폼 이미지 빌드 가능

2. **Week 2: Performance (US3 + US4)**
   - Complete T021-T033
   - Deliverable: 빌드 시간 50% 단축, 이미지 크기 <100MB
   - Value: 프로덕션 품질 이미지 (빠른 빌드, 작은 크기)

3. **Week 3: Automation (US5 + Polish)**
   - Complete T034-T054
   - Deliverable: 완전 자동화된 CI/CD 파이프라인
   - Value: main 병합 시 자동 Docker Hub 배포

### Testing Strategy

**Per User Story**:
- US1: `docker buildx build --platform ... --load .` 성공 확인
- US2: `file /usr/local/bin/backend` 아키텍처 검증
- US3: 빌드 시간 비교 (첫 빌드 vs 재빌드)
- US4: `docker images` 크기 확인 (<100MB)
- US5: GitHub Actions 로그 확인, Docker Hub 이미지 확인

**Integration Test** (최종):
```bash
# 1. 완전한 멀티플랫폼 빌드
docker buildx build --platform linux/amd64,linux/arm64,linux/arm/v7 \
  --tag username/realtime-svg-backend:test \
  --push .

# 2. ARM64 서버에서 pull 및 실행
docker pull --platform linux/arm64 username/realtime-svg-backend:test
docker run --rm username/realtime-svg-backend:test --version

# 3. 이미지 크기 및 매니페스트 확인
docker manifest inspect username/realtime-svg-backend:test
```

---

## Task Summary

**Total Tasks**: 54  
**Tasks per User Story**:
- Setup: 3 tasks
- Foundational: 4 tasks (blocking)
- US1 (P1): 7 tasks 🎯 MVP
- US2 (P1): 6 tasks
- US3 (P2): 6 tasks
- US4 (P2): 7 tasks
- US5 (P3): 15 tasks
- Polish: 6 tasks

**Parallel Opportunities**: 18 tasks marked [P] (33% 병렬 가능)

**Suggested MVP Scope**: 
- Phase 1 (Setup) + Phase 2 (Foundational) + Phase 3 (US1) = **14 tasks**
- Estimated time: 2-3 days
- Deliverable: 로컬에서 멀티플랫폼 Docker 이미지 빌드 성공

**Format Validation**: ✅ All tasks follow checklist format (checkbox, ID, [P]/[Story] labels, file paths)
