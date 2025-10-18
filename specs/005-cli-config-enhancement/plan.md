# Implementation Plan: CLI Configuration Enhancement

**Branch**: `005-cli-config-enhancement` | **Date**: 2025-10-18 | **Spec**: [spec.md](./spec.md)  
**Input**: Feature specification from `/specs/005-cli-config-enhancement/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

현재 환경 변수만으로 받는 CLI 설정을 clap, figment, dotenvy를 사용하여 고도화한다. 세 가지 설정 소스(커맨드라인 옵션, 환경 변수, 설정 파일)를 계층적으로 병합하여 운영자가 유연하게 서버를 구성할 수 있게 한다. 우선순위는 CLI > ENV > Config 순서이며, --config 옵션으로 설정 파일 경로를 지정할 수 있다.

## Technical Context

**Language/Version**: Rust 1.90.0 (edition 2021)  
**Primary Dependencies**: clap 4.5 (CLI parsing), figment 0.10 (config merging), dotenvy 0.15 (.env loading), serde 1.0 (serialization)  
**Storage**: N/A (configuration only, existing Redis connection managed by AppState)  
**Testing**: cargo test (workspace-level unit/integration tests)  
**Target Platform**: Linux server (tokio async runtime)  
**Project Type**: Multi-crate workspace (common/backend/frontend)  
**Performance Goals**: 설정 로딩은 서버 시작 시 한 번만 수행되므로 성능 목표 없음 (< 100ms 초기화 시간은 충분)  
**Constraints**: 
- 기존 환경 변수 기반 설정과 100% 호환성 유지 (REDIS_URL 등)
- 단일 파일 수정으로 마이그레이션 완료 (main.rs만 변경)
- 설정 검증 실패 시 명확한 오류 메시지 제공 (panic 대신 anyhow::Result 반환)
**Scale/Scope**: 
- 설정 항목: 5개 이하 (redis_url, host, port, log_level 등)
- 지원 형식: YAML만 (JSON/TOML은 out of scope)
- 파일 크기: < 10KB (설정 파일)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Initial Compliance Review

| Principle | Status | Notes |
|-----------|--------|-------|
| **I. Workspace Modularity** | ✅ PASS | 설정 구조체는 `common` 크레이트에 정의, backend main.rs에서만 로딩 로직 구현. 프런트엔드는 영향 없음. |
| **II. Contract-First API Design** | ✅ PASS | CLI 설정은 내부 로직이므로 API 계약 불필요. 기존 HTTP 엔드포인트 변경 없음. |
| **III. Template-Based SVG Rendering** | ✅ PASS | SVG 템플릿 로직은 변경되지 않음. 설정 시스템만 개선. |
| **IV. Testing Discipline** | ⚠️ REQUIRES ATTENTION | 설정 로딩 로직에 대한 유닛 테스트 필요. 우선순위 병합 로직 검증 필수. |
| **V. Observability & Debugging** | ✅ PASS | FR-012: 시작 시 최종 설정값을 로그 출력 (민감 정보 마스킹). tracing 사용. |
| **VI. Simplicity & Incremental Delivery** | ✅ PASS | 기존 코드에서 최소한의 변경. main.rs의 환경 변수 읽기를 설정 구조체 로딩으로 교체. |

**Gate Decision**: ✅ PROCEED to Phase 0 research

### Testing Requirements (IV)
- [ ] 설정 병합 우선순위 유닛 테스트 (CLI > ENV > Config)
- [ ] 설정 파일 파싱 실패 시 오류 메시지 검증
- [ ] 바이너리 경로 해석 로직 테스트
- [ ] 기본값 fallback 테스트

## Project Structure

### Documentation (this feature)

```
specs/005-cli-config-enhancement/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (N/A for this feature - no API changes)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```
crates/
├── common/
│   └── src/
│       ├── config.rs         # NEW: 설정 구조체 및 병합 로직
│       └── lib.rs            # MODIFIED: pub mod config 추가
├── backend/
│   └── src/
│       └── main.rs           # MODIFIED: 환경 변수 읽기 → Config::load() 호출
└── frontend/                 # NO CHANGES

tests/
└── config_test.rs            # NEW: 설정 로딩 및 우선순위 통합 테스트

config.yaml                   # NEW: 예제 설정 파일 (문서용)
.env.example                  # NEW: 예제 .env 파일 (문서용)
```

**Structure Decision**: Rust workspace 구조를 유지하며 설정 로직은 `common` 크레이트에 배치하여 재사용 가능하도록 한다. Backend main.rs는 설정 로딩만 담당하고, 검증 및 병합 로직은 common::config 모듈에서 처리한다.

## Complexity Tracking

*이 기능은 Constitution Check 위반 없음. 테이블 비어 있음.*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |

---

## Post-Design Constitution Check

*Re-evaluation after Phase 1 design (research.md, data-model.md, quickstart.md completed)*

| Principle | Status | Notes |
|-----------|--------|-------|
| **I. Workspace Modularity** | ✅ PASS | 설계 확인: common::config 모듈이 독립적으로 정의됨. Backend만 의존, frontend 영향 없음. |
| **II. Contract-First API Design** | ✅ PASS | API 변경 없음. 내부 설정 로직만 개선. |
| **III. Template-Based SVG Rendering** | ✅ PASS | 템플릿 시스템 변경 없음. |
| **IV. Testing Discipline** | ✅ PASS | data-model.md에 테스트 시나리오 정의됨. 유닛/통합 테스트 계획 완료. |
| **V. Observability & Debugging** | ✅ PASS | research.md #6: 민감 정보 마스킹 Debug trait 설계 완료. tracing::info! 사용. |
| **VI. Simplicity & Incremental Delivery** | ✅ PASS | quickstart.md 시나리오 A-C: 점진적 마이그레이션 경로 제공. Breaking changes 없음. |

**Final Gate Decision**: ✅ ALL GATES PASSED - Ready for `/speckit.tasks`

### Testing Discipline Update (IV)

Phase 1 설계 완료 후 테스트 요구사항:

- [x] 설정 병합 우선순위 유닛 테스트 설계 완료 (data-model.md "Testing Scenarios")
- [x] 설정 파일 파싱 오류 테스트 설계 완료 (data-model.md "Error States")
- [x] 바이너리 경로 해석 로직 설계 완료 (research.md #4)
- [x] 기본값 fallback 테스트 설계 완료 (data-model.md "Default Values")

**Next Phase**: `/speckit.tasks` 명령으로 Phase 2 태스크 분해 진행 가능.
