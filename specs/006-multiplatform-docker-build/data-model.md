# Data Model: 멀티플랫폼 Docker 빌드 설정

**Feature**: [spec.md](./spec.md)  
**Plan**: [plan.md](./plan.md)  
**Date**: 2025-01-18

## Overview

이 기능은 애플리케이션 런타임 데이터 모델을 변경하지 않는다. 대신 빌드 프로세스와 CI/CD 파이프라인에 관련된 구성 데이터를 정의한다. 이 문서는 Docker 빌드 시스템에서 사용되는 주요 엔티티와 그 관계를 설명한다.

---

## Build Configuration Entities

### 1. Platform Target

빌드 대상 플랫폼을 정의하는 엔티티.

**Attributes**:
- `platform_id` (String): Docker 플랫폼 식별자 (예: `linux/amd64`, `linux/arm64`, `linux/arm/v7`)
- `rust_target` (String): Rust 타겟 트리플 (예: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`)
- `cross_compiler` (String): 크로스 컴파일러 패키지명 (예: `gcc-aarch64-linux-gnu`)
- `linker_env_var` (String): Cargo 링커 환경 변수명 (예: `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER`)
- `linker_binary` (String): 링커 실행 파일명 (예: `aarch64-linux-gnu-gcc`)

**Validation Rules**:
- `platform_id`는 지원되는 플랫폼 목록 (`linux/amd64`, `linux/arm64`, `linux/arm/v7`) 중 하나여야 한다.
- `rust_target`은 유효한 Rust 타겟 트리플이어야 한다.
- `cross_compiler`는 Debian 패키지 저장소에서 사용 가능해야 한다.

**State Transitions**: 없음 (정적 구성)

**Example**:
```yaml
platform: linux/arm64
rust_target: aarch64-unknown-linux-gnu
cross_compiler: gcc-aarch64-linux-gnu
linker_env_var: CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER
linker_binary: aarch64-linux-gnu-gcc
```

---

### 2. Build Target

빌드할 바이너리 타겟을 정의하는 엔티티.

**Attributes**:
- `target_name` (String): 빌드 타겟명 (`backend`, `frontend`)
- `dockerfile_path` (String): Dockerfile 경로 (예: `./Dockerfile`, `./Dockerfile.frontend`)
- `binary_name` (String): 컴파일된 바이너리 이름 (예: `backend`, `frontend`)
- `binary_path` (String): 빌드 후 바이너리 경로 (예: `/app/target/release/backend`)
- `runtime_dependencies` (List<String>): 런타임 시스템 패키지 목록 (예: `["ca-certificates", "tini"]`)

**Validation Rules**:
- `target_name`은 Cargo.toml의 `[[bin]]` 섹션에 정의된 바이너리명과 일치해야 한다.
- `dockerfile_path`는 존재하는 파일 경로여야 한다.
- `runtime_dependencies`의 모든 패키지는 Debian 저장소에서 사용 가능해야 한다.

**Relationships**:
- `Build Target` → (1:N) → `Platform Target`: 각 빌드 타겟은 여러 플랫폼용으로 빌드된다.

**Example**:
```yaml
target_name: backend
dockerfile_path: ./Dockerfile
binary_name: backend
binary_path: /app/target/release/backend
runtime_dependencies:
  - ca-certificates
  - tini
```

---

### 3. Docker Build Context

Docker 빌드 실행 시 사용되는 컨텍스트 구성.

**Attributes**:
- `context_path` (String): 빌드 컨텍스트 루트 경로 (예: `.`)
- `dockerfile` (String): Dockerfile 경로 (상대 또는 절대)
- `platforms` (List<String>): 빌드할 플랫폼 목록 (예: `["linux/amd64", "linux/arm64"]`)
- `tags` (List<String>): 이미지 태그 목록 (예: `["username/app:latest", "username/app:v1.0.0"]`)
- `build_args` (Map<String, String>): 빌드 시 전달할 아규먼트 (예: `{"RUST_VERSION": "1.86"}`)
- `cache_from` (String): 캐시 소스 (예: `type=gha`)
- `cache_to` (String): 캐시 대상 (예: `type=gha,mode=max`)

**Validation Rules**:
- `context_path`는 유효한 디렉토리여야 한다.
- `platforms`는 최소 1개 이상의 플랫폼을 포함해야 한다.
- `tags`는 유효한 Docker 이미지 태그 형식이어야 한다 (예: `[registry/]repository[:tag]`).

**Example**:
```yaml
context_path: .
dockerfile: ./Dockerfile
platforms:
  - linux/amd64
  - linux/arm64
  - linux/arm/v7
tags:
  - username/realtime-svg-backend:latest
  - username/realtime-svg-backend:sha-abc123
build_args:
  RUST_VERSION: "1.86"
cache_from: type=gha
cache_to: type=gha,mode=max
```

---

### 4. CI/CD Workflow Configuration

GitHub Actions 워크플로우 설정을 정의하는 엔티티.

**Attributes**:
- `workflow_name` (String): 워크플로우 이름 (예: `Docker Multiplatform Build`)
- `trigger_events` (List<String>): 워크플로우 트리거 이벤트 (예: `["push", "pull_request"]`)
- `trigger_branches` (List<String>): 트리거 브랜치 (예: `["main"]`)
- `docker_registry` (String): Docker 레지스트리 URL (예: `docker.io`)
- `registry_username_secret` (String): 레지스트리 사용자명 시크릿 이름 (예: `DOCKERHUB_USERNAME`)
- `registry_token_secret` (String): 레지스트리 토큰 시크릿 이름 (예: `DOCKERHUB_TOKEN`)
- `push_on_pr` (Boolean): PR 시 이미지 푸시 여부 (기본값: `false`)

**Validation Rules**:
- `trigger_events`는 유효한 GitHub Actions 이벤트명이어야 한다.
- `registry_username_secret`과 `registry_token_secret`은 GitHub Secrets에 설정되어야 한다.

**Example**:
```yaml
workflow_name: Docker Multiplatform Build
trigger_events:
  - push
  - pull_request
trigger_branches:
  - main
docker_registry: docker.io
registry_username_secret: DOCKERHUB_USERNAME
registry_token_secret: DOCKERHUB_TOKEN
push_on_pr: false
```

---

### 5. Build Cache Configuration

Docker 빌드 캐시 전략을 정의하는 엔티티.

**Attributes**:
- `cache_type` (Enum): 캐시 백엔드 유형 (`gha`, `registry`, `local`, `inline`)
- `cache_scope` (String): 캐시 스코프 (예: `buildkit-cache`)
- `max_cache_size` (Integer): 최대 캐시 크기 (바이트, GHA는 10GB 제한)
- `cache_mode` (Enum): 캐시 모드 (`min`, `max`)
  - `min`: 최종 이미지 레이어만 캐시
  - `max`: 모든 중간 레이어 캐시 (권장)

**Validation Rules**:
- `cache_type`이 `gha`일 때 `max_cache_size`는 10GB를 초과할 수 없다.
- `cache_mode`는 `min` 또는 `max`여야 한다.

**Example**:
```yaml
cache_type: gha
cache_scope: buildkit-multiplatform-backend
max_cache_size: 10737418240  # 10GB
cache_mode: max
```

---

## Entity Relationships

```
Build Target (1:N) Platform Target
   │
   ├─── docker build command
   │       │
   │       ├─── Docker Build Context
   │       │       │
   │       │       └─── Build Cache Configuration
   │       │
   │       └─── Platform Target (N)
   │
   └─── deployed via CI/CD Workflow Configuration
```

**설명**:
1. 각 `Build Target` (backend, frontend)은 여러 `Platform Target`용으로 빌드된다.
2. `Docker Build Context`는 빌드 실행 시 사용되는 모든 설정을 포함한다.
3. `Build Cache Configuration`은 빌드 성능을 최적화하기 위한 캐싱 전략을 정의한다.
4. `CI/CD Workflow Configuration`은 GitHub Actions에서 자동화된 빌드 및 배포를 관리한다.

---

## Configuration Files

### Dockerfile

**Purpose**: 멀티플랫폼 Docker 이미지를 빌드하기 위한 선언적 정의.

**Key Sections**:
- **Builder Stage**: Rust 컴파일러와 크로스 컴파일 도구 설정
- **Planner Stage**: cargo-chef를 사용한 종속성 그래프 생성
- **Cook Stage**: 종속성 빌드 (캐시 레이어)
- **Build Stage**: 애플리케이션 빌드
- **Runtime Stage**: 최소 런타임 이미지

**Platform-Specific Logic**:
```dockerfile
ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
      "linux/amd64") echo x86_64-unknown-linux-gnu > /rust_target.txt ;; \
      "linux/arm64") echo aarch64-unknown-linux-gnu > /rust_target.txt ;; \
      "linux/arm/v7") echo armv7-unknown-linux-gnueabihf > /rust_target.txt ;; \
      *) echo "Unsupported platform" && exit 1 ;; \
    esac
```

---

### .dockerignore

**Purpose**: 빌드 컨텍스트에서 제외할 파일 및 디렉토리 정의.

**Excluded Items**:
- `target/` - Cargo 빌드 아티팩트
- `.git/` - Git 메타데이터
- `specs/` - 사양 문서
- `*.md` - 문서 파일 (README.md 제외 가능)
- `.env*` - 환경 변수 파일

---

### GitHub Actions Workflow (`.github/workflows/docker-build.yml`)

**Purpose**: CI/CD 파이프라인에서 멀티플랫폼 이미지를 자동으로 빌드하고 푸시.

**Key Steps**:
1. Checkout 코드
2. QEMU 설정 (ARM 에뮬레이션)
3. Docker Buildx 설정 (멀티플랫폼 빌드)
4. Docker Hub 로그인
5. 빌드 및 푸시 (각 플랫폼별)

**Secrets Required**:
- `DOCKERHUB_USERNAME`: Docker Hub 사용자명
- `DOCKERHUB_TOKEN`: Docker Hub 액세스 토큰

---

## Platform Target Mapping Table

| Docker Platform | Rust Target Triple | Cross Compiler | Linker Env Var | Linker Binary |
|-----------------|---------------------|----------------|----------------|---------------|
| `linux/amd64` | `x86_64-unknown-linux-gnu` | `gcc-x86-64-linux-gnu` | `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER` | `x86_64-linux-gnu-gcc` |
| `linux/arm64` | `aarch64-unknown-linux-gnu` | `gcc-aarch64-linux-gnu` | `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER` | `aarch64-linux-gnu-gcc` |
| `linux/arm/v7` | `armv7-unknown-linux-gnueabihf` | `gcc-arm-linux-gnueabihf` | `CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER` | `arm-linux-gnueabihf-gcc` |

---

## Notes

- 이 데이터 모델은 빌드 시스템과 CI/CD 구성에만 적용된다.
- 런타임 애플리케이션 데이터 모델 (SessionData, SvgFrame 등)은 변경되지 않는다.
- 모든 구성 값은 Dockerfile, GitHub Actions 워크플로우, 또는 빌드 스크립트에 선언적으로 정의된다.
