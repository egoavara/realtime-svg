# Feature Specification: 멀티플랫폼 Docker 빌드 설정

**Feature Branch**: `006-multiplatform-docker-build`  
**Created**: 2025-01-18  
**Status**: Draft  
**Input**: User description: "멀티플랫폼 Docker 빌드 설정"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - 멀티플랫폼 이미지 빌드 (Priority: P1)

개발자가 단일 명령으로 AMD64, ARM64, ARMv7 플랫폼용 Docker 이미지를 동시에 빌드하여 Docker Hub 또는 컨테이너 레지스트리에 푸시할 수 있다.

**Why this priority**: 멀티플랫폼 지원은 다양한 아키텍처(클라우드 서버, Raspberry Pi, ARM 기반 서버)에서 애플리케이션을 배포할 수 있게 하는 핵심 기능이다.

**Independent Test**: `docker buildx build --platform linux/amd64,linux/arm64,linux/arm/v7 .` 명령 실행 후 각 플랫폼별 이미지가 레지스트리에 올바르게 생성되는지 확인한다.

**Acceptance Scenarios**:

1. **Given** Docker Buildx가 설치되어 있고, **When** 개발자가 멀티플랫폼 빌드 명령을 실행하면, **Then** AMD64, ARM64, ARMv7용 이미지가 성공적으로 빌드된다.
2. **Given** 빌드된 멀티플랫폼 이미지가 레지스트리에 푸시되었을 때, **When** ARM64 서버에서 이미지를 pull하면, **Then** 올바른 ARM64 이미지가 자동으로 다운로드되어 실행된다.
3. **Given** 멀티플랫폼 빌드 과정에서, **When** 하나의 플랫폼에서 빌드 오류가 발생하면, **Then** 명확한 오류 메시지와 함께 빌드가 중단된다.

---

### User Story 2 - Rust 크로스 컴파일 최적화 (Priority: P1)

Dockerfile이 Rust 프로젝트를 각 플랫폼별 타겟(x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu 등)으로 크로스 컴파일하여 올바른 바이너리를 생성한다.

**Why this priority**: Rust 프로젝트의 멀티플랫폼 빌드는 크로스 컴파일러 및 링커 설정이 필수이며, 잘못된 설정 시 빌드가 실패하거나 실행 불가능한 바이너리가 생성된다.

**Independent Test**: 각 플랫폼용 빌드 컨테이너에서 `file /app/backend` 명령으로 바이너리가 올바른 아키텍처용으로 컴파일되었는지 확인한다.

**Acceptance Scenarios**:

1. **Given** TARGETPLATFORM이 linux/arm64로 설정되었을 때, **When** Dockerfile의 builder 단계가 실행되면, **Then** aarch64-unknown-linux-gnu 타겟으로 바이너리가 컴파일된다.
2. **Given** 멀티 크레이트 워크스페이스 구조일 때, **When** Dockerfile이 빌드되면, **Then** 모든 크레이트 종속성이 올바르게 크로스 컴파일된다.
3. **Given** ARMv7 플랫폼 빌드 시, **When** 적절한 링커(arm-linux-gnueabihf-gcc)가 설정되면, **Then** 빌드가 성공적으로 완료된다.

---

### User Story 3 - 빌드 캐싱 최적화 (Priority: P2)

Dockerfile의 레이어 캐싱을 활용하여 종속성 빌드 시간을 단축하고, 소스 코드 변경 시에만 애플리케이션을 다시 빌드한다.

**Why this priority**: Rust 프로젝트는 종속성 컴파일 시간이 길기 때문에, 효율적인 캐싱 전략은 개발 속도를 크게 향상시킨다.

**Independent Test**: Cargo.toml 변경 없이 소스 코드만 수정한 후 재빌드 시 종속성 빌드 단계가 캐시에서 로드되는지 확인한다.

**Acceptance Scenarios**:

1. **Given** Cargo.toml과 Cargo.lock이 변경되지 않았을 때, **When** 소스 코드만 수정하여 재빌드하면, **Then** 종속성 빌드 단계가 스킵되고 캐시가 사용된다.
2. **Given** 새로운 crate가 Cargo.toml에 추가되었을 때, **When** 재빌드하면, **Then** 종속성 레이어가 무효화되고 재빌드된다.
3. **Given** Docker Buildx 캐시가 설정되어 있을 때, **When** 멀티플랫폼 빌드를 실행하면, **Then** 각 플랫폼별 캐시가 독립적으로 관리된다.

---

### User Story 4 - 최종 이미지 크기 최소화 (Priority: P2)

멀티 스테이지 빌드를 사용하여 최종 런타임 이미지에는 컴파일된 바이너리와 필수 런타임 라이브러리만 포함시켜 이미지 크기를 최소화한다.

**Why this priority**: 작은 이미지 크기는 배포 속도를 향상시키고 저장 공간을 절약한다.

**Independent Test**: `docker images` 명령으로 최종 이미지 크기가 100MB 이하인지 확인한다.

**Acceptance Scenarios**:

1. **Given** 멀티 스테이지 빌드가 완료되었을 때, **When** 최종 이미지를 검사하면, **Then** Rust 컴파일러, 빌드 도구, 소스 코드가 포함되지 않는다.
2. **Given** debian:bookworm-slim 베이스 이미지를 사용할 때, **When** 최종 이미지를 빌드하면, **Then** 이미지 크기가 50MB 이하를 유지한다.
3. **Given** 바이너리가 동적 링킹으로 빌드되었을 때, **When** 런타임 컨테이너에서 실행하면, **Then** 필요한 모든 공유 라이브러리(glibc, ca-certificates)가 포함되어 있다.

---

### User Story 5 - GitHub Actions CI/CD 통합 (Priority: P3)

GitHub Actions 워크플로우에서 멀티플랫폼 Docker 이미지를 자동으로 빌드하고 Docker Hub에 푸시한다.

**Why this priority**: CI/CD 자동화는 수동 배포 작업을 제거하고 지속적인 배포를 가능하게 한다.

**Independent Test**: PR이 main 브랜치에 병합되었을 때 GitHub Actions 워크플로우가 성공적으로 실행되고 이미지가 푸시되는지 확인한다.

**Acceptance Scenarios**:

1. **Given** GitHub Actions 워크플로우가 구성되어 있을 때, **When** main 브랜치에 커밋이 푸시되면, **Then** 멀티플랫폼 빌드가 자동으로 실행된다.
2. **Given** Docker Hub 자격 증명이 GitHub Secrets에 저장되어 있을 때, **When** 빌드가 완료되면, **Then** 이미지가 자동으로 레지스트리에 푸시된다.
3. **Given** 빌드 실패 시, **When** GitHub Actions 워크플로우가 종료되면, **Then** 개발자에게 알림이 전송되고 실패 로그를 확인할 수 있다.

---

### Edge Cases

- TARGETPLATFORM 환경 변수가 지원되지 않는 플랫폼(예: linux/riscv64)으로 설정되었을 때?
- 크로스 컴파일 중 링커를 찾을 수 없을 때?
- 종속성 중 하나가 특정 플랫폼에서 컴파일 실패할 때?
- Docker Buildx가 설치되지 않은 환경에서 빌드 시도 시?
- 네트워크 오류로 crates.io에서 종속성 다운로드 실패 시?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Dockerfile은 linux/amd64, linux/arm64, linux/arm/v7 플랫폼을 지원해야 한다.
- **FR-002**: 빌드 시스템은 BUILDPLATFORM과 TARGETPLATFORM 인자를 활용하여 적절한 Rust 타겟을 자동으로 선택해야 한다.
- **FR-003**: 각 플랫폼별로 올바른 크로스 컴파일 링커(gcc-aarch64-linux-gnu, gcc-arm-linux-gnueabihf)가 설치되어야 한다.
- **FR-004**: Cargo 종속성 빌드와 애플리케이션 빌드를 분리하여 Docker 레이어 캐싱을 최적화해야 한다.
- **FR-005**: 멀티 스테이지 빌드를 사용하여 최종 이미지에는 컴파일된 바이너리만 포함해야 한다.
- **FR-006**: 최종 런타임 이미지는 debian:bookworm-slim 베이스를 사용하며 tini와 ca-certificates를 포함해야 한다.
- **FR-007**: 빌드 실패 시 명확한 오류 메시지와 함께 종료되어야 한다.
- **FR-008**: backend와 frontend 바이너리를 각각 독립적으로 빌드할 수 있어야 한다.
- **FR-009**: .dockerignore 파일을 사용하여 불필요한 파일(target/, .git/, specs/)을 빌드 컨텍스트에서 제외해야 한다.

### Key Entities

- **Docker 빌드 스테이지**: 
  - Builder 스테이지: Rust 컴파일러와 크로스 컴파일 도구가 포함된 빌드 환경
  - Runtime 스테이지: 컴파일된 바이너리를 실행하는 최소 환경

- **플랫폼 타겟 매핑**:
  - linux/amd64 → x86_64-unknown-linux-gnu
  - linux/arm64 → aarch64-unknown-linux-gnu
  - linux/arm/v7 → armv7-unknown-linux-gnueabihf

- **빌드 아티팩트**:
  - 컴파일된 바이너리: /app/backend (또는 /app/frontend)
  - 종속성 캐시: target/[RUST_TARGET]/release/deps/

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 단일 빌드 명령으로 3개 플랫폼(AMD64, ARM64, ARMv7)용 이미지가 5분 이내에 생성된다.
- **SC-002**: 소스 코드만 변경 후 재빌드 시 종속성 캐시 활용으로 빌드 시간이 50% 이상 단축된다.
- **SC-003**: 최종 런타임 이미지 크기가 각 플랫폼당 100MB 이하를 유지한다.
- **SC-004**: 빌드된 이미지가 각 플랫폼에서 정상적으로 실행되고 헬스체크를 통과한다.
- **SC-005**: GitHub Actions에서 자동 빌드 및 푸시가 10분 이내에 완료된다.
