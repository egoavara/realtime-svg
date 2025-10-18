# Dockerfile Contract

**Feature**: [spec.md](../spec.md)  
**Date**: 2025-01-18

## Purpose

이 문서는 멀티플랫폼 Docker 빌드를 위한 Dockerfile의 구조, 인터페이스, 동작을 정의한다. Dockerfile은 선언적 빌드 명세로, 입력(소스 코드, 빌드 아규먼트)과 출력(Docker 이미지)의 계약을 명시한다.

---

## Build Arguments (Input Contract)

Dockerfile은 다음 빌드 아규먼트를 입력으로 받는다:

### Platform Arguments (Docker BuildKit 자동 제공)

| Argument | Type | Description | Example |
|----------|------|-------------|---------|
| `BUILDPLATFORM` | String | 빌드를 실행하는 플랫폼 | `linux/amd64` |
| `TARGETPLATFORM` | String | 최종 이미지가 실행될 플랫폼 | `linux/arm64` |
| `TARGETOS` | String | 타겟 운영체제 | `linux` |
| `TARGETARCH` | String | 타겟 아키텍처 | `arm64` |

### Custom Build Arguments

| Argument | Type | Default | Description |
|----------|------|---------|-------------|
| `RUST_VERSION` | String | `1.86` | Rust 컴파일러 버전 |
| `TARGET_BINARY` | String | `backend` | 빌드할 바이너리 이름 (`backend` 또는 `frontend`) |

---

## Build Stages (Transformation Contract)

### Stage 1: Chef (Base)

**Purpose**: cargo-chef 도구가 포함된 기본 이미지 준비.

**Input**: 없음  
**Output**: cargo-chef 설치된 Rust 이미지  
**Base Image**: `lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION}`

```dockerfile
FROM lukemathwalker/cargo-chef:latest-rust-1.86 AS chef
WORKDIR /app
```

---

### Stage 2: Planner

**Purpose**: Cargo 종속성 그래프 분석 및 recipe.json 생성.

**Input**:
- 전체 소스 코드 (COPY . .)
- Cargo.toml, Cargo.lock

**Output**:
- `/app/recipe.json` - 종속성 그래프 정의

**Contract**:
```dockerfile
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json
```

**Guarantees**:
- `recipe.json`은 워크스페이스의 모든 크레이트 종속성을 포함한다.
- 소스 코드 변경이 recipe.json에 영향을 주지 않으면 캐시가 유지된다.

---

### Stage 3: Builder (Cross-Compilation Setup)

**Purpose**: 크로스 컴파일 환경 설정 및 종속성 빌드.

**Input**:
- `TARGETPLATFORM` (ARG)
- `/app/recipe.json` (from planner stage)

**Output**:
- Rust 타겟 추가 완료
- 크로스 컴파일러 설치 완료
- 종속성 빌드 완료 (`target/${RUST_TARGET}/release/deps/`)

**Contract**:

```dockerfile
FROM --platform=$BUILDPLATFORM rust:${RUST_VERSION}-bookworm AS builder

ARG TARGETPLATFORM

# 1. Platform → Rust target 매핑
RUN case "$TARGETPLATFORM" in \
      "linux/amd64") echo x86_64-unknown-linux-gnu > /rust_target.txt ;; \
      "linux/arm64") echo aarch64-unknown-linux-gnu > /rust_target.txt ;; \
      "linux/arm/v7") echo armv7-unknown-linux-gnueabihf > /rust_target.txt ;; \
      *) echo "ERROR: Unsupported platform $TARGETPLATFORM" && exit 1 ;; \
    esac

# 2. Rust 타겟 추가
RUN export RUST_TARGET=$(cat /rust_target.txt) && \
    echo "Adding Rust target: $RUST_TARGET" && \
    rustup target add $RUST_TARGET

# 3. 크로스 컴파일러 설치
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc-aarch64-linux-gnu \
    gcc-arm-linux-gnueabihf \
    gcc-x86-64-linux-gnu \
    libc6-dev-arm64-cross \
    libc6-dev-armhf-cross \
    libc6-dev-amd64-cross \
    && rm -rf /var/lib/apt/lists/*

# 4. cargo-chef로 종속성 빌드
COPY --from=planner /app/recipe.json recipe.json
RUN export RUST_TARGET=$(cat /rust_target.txt) && \
    case "$RUST_TARGET" in \
      "aarch64-unknown-linux-gnu") \
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc ;; \
      "armv7-unknown-linux-gnueabihf") \
        export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc ;; \
    esac && \
    cargo chef cook --release --target $RUST_TARGET --recipe-path recipe.json

# 5. 소스 코드 복사 및 최종 빌드
COPY . .
RUN export RUST_TARGET=$(cat /rust_target.txt) && \
    case "$RUST_TARGET" in \
      "aarch64-unknown-linux-gnu") \
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc ;; \
      "armv7-unknown-linux-gnueabihf") \
        export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc ;; \
    esac && \
    cargo build --release --target $RUST_TARGET --bin ${TARGET_BINARY} && \
    mv target/$RUST_TARGET/release/${TARGET_BINARY} /app/${TARGET_BINARY}
```

**Guarantees**:
- 지원되지 않는 `TARGETPLATFORM`은 exit 1로 빌드를 중단한다.
- 올바른 아키텍처용 바이너리가 `/app/${TARGET_BINARY}`에 생성된다.
- Cargo.toml 변경 없이 소스만 수정 시 종속성 빌드 단계가 캐시된다.

**Error Handling**:
- Unsupported platform → `exit 1` with error message
- Missing linker → `error: linker 'cc' not found`
- Compilation failure → Cargo error output with file/line info

---

### Stage 4: Runtime

**Purpose**: 최소 런타임 이미지 생성.

**Input**:
- `/app/${TARGET_BINARY}` (from builder stage)

**Output**:
- 최종 Docker 이미지 (debian:bookworm-slim 기반)
- 바이너리: `/usr/local/bin/${TARGET_BINARY}`

**Contract**:

```dockerfile
FROM debian:bookworm-slim AS runtime

# 런타임 종속성 설치
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    tini \
    && rm -rf /var/lib/apt/lists/*

# 바이너리 복사
ARG TARGET_BINARY=backend
COPY --from=builder /app/${TARGET_BINARY} /usr/local/bin/${TARGET_BINARY}

# 실행 권한 부여
RUN chmod +x /usr/local/bin/${TARGET_BINARY}

# tini를 PID 1로 사용
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/usr/local/bin/${TARGET_BINARY}"]
```

**Guarantees**:
- 이미지 크기: <100MB
- 런타임 종속성: `ca-certificates`, `tini`만 포함
- 바이너리는 실행 가능하며 올바른 아키텍처용으로 컴파일됨
- PID 1 시그널 처리가 올바르게 작동 (tini 사용)

**Runtime Dependencies**:
- `ca-certificates`: TLS/HTTPS 연결 (Redis TLS, 외부 API)
- `tini`: 좀비 프로세스 정리 및 시그널 전달

---

## Build Command Contract

### Local Build (Single Platform)

```bash
docker build \
  --build-arg TARGET_BINARY=backend \
  --tag realtime-svg-backend:latest \
  .
```

**Expected Result**:
- 현재 플랫폼용 이미지 생성
- 빌드 시간: ~3-5분 (종속성 캐시 없을 때)
- 빌드 시간: ~1-2분 (종속성 캐시 있을 때)

---

### Multiplatform Build

```bash
docker buildx build \
  --platform linux/amd64,linux/arm64,linux/arm/v7 \
  --build-arg TARGET_BINARY=backend \
  --tag username/realtime-svg-backend:latest \
  --push \
  .
```

**Expected Result**:
- 3개 플랫폼용 이미지 생성 및 푸시
- 빌드 시간: ~5-8분 (종속성 캐시 없을 때)
- 빌드 시간: ~2-3분 (종속성 캐시 있을 때)
- 매니페스트 리스트 생성: 클라이언트가 플랫폼에 맞는 이미지를 자동 선택

---

## Output Contract

### Image Metadata

**Labels** (권장):
```dockerfile
LABEL org.opencontainers.image.title="realtime-svg-backend"
LABEL org.opencontainers.image.description="Multiplatform Rust backend for realtime-svg"
LABEL org.opencontainers.image.version="${VERSION}"
LABEL org.opencontainers.image.platform="${TARGETPLATFORM}"
```

**Environment Variables**:
```dockerfile
ENV PATH="/usr/local/bin:${PATH}"
```

---

### Image Verification

**Check Binary Architecture**:
```bash
docker run --rm --entrypoint file username/realtime-svg-backend /usr/local/bin/backend
```

**Expected Output**:
- AMD64: `ELF 64-bit LSB executable, x86-64, version 1 (SYSV)`
- ARM64: `ELF 64-bit LSB executable, ARM aarch64, version 1 (SYSV)`
- ARMv7: `ELF 32-bit LSB executable, ARM, EABI5 version 1 (SYSV)`

**Check Dynamic Dependencies**:
```bash
docker run --rm --entrypoint ldd username/realtime-svg-backend /usr/local/bin/backend
```

**Expected Output**:
```
linux-vdso.so.1 (0x...)
libc.so.6 => /lib/.../libc.so.6 (0x...)
libgcc_s.so.1 => /lib/.../libgcc_s.so.1 (0x...)
/lib/ld-linux-....so.2 (0x...)
```

**Check Image Size**:
```bash
docker images username/realtime-svg-backend:latest
```

**Expected Size**: <100MB

---

## Error Scenarios

| Error | Cause | Resolution |
|-------|-------|------------|
| `Unsupported platform: linux/riscv64` | TARGETPLATFORM이 지원 목록에 없음 | 지원되는 플랫폼 사용 (amd64, arm64, arm/v7) |
| `linker 'cc' not found` | 크로스 컴파일러 미설치 | `gcc-*-linux-gnu` 패키지 설치 확인 |
| `cannot find -lgcc_s` | libc 개발 파일 미설치 | `libc6-dev-*-cross` 패키지 설치 확인 |
| `error: no bin target named 'frontend'` | TARGET_BINARY가 Cargo.toml에 없음 | 올바른 바이너리명 사용 또는 Cargo.toml 수정 |
| Binary won't execute in runtime | 아키텍처 불일치 | `file` 명령으로 바이너리 아키텍처 검증 |

---

## Performance Guarantees

| Scenario | Expected Build Time | Cache Efficiency |
|----------|---------------------|------------------|
| 첫 빌드 (캐시 없음) | 5-8분 | N/A |
| Cargo.toml 변경 없이 소스만 수정 | 1-2분 | 종속성 레이어 캐시됨 |
| Cargo.toml에 새 crate 추가 | 3-5분 | Planner 레이어만 무효화 |
| Dockerfile만 수정 | 5-8분 | 캐시 무효화 |

---

## Compliance

이 Dockerfile은 다음 표준을 준수한다:

- **OCI Image Spec**: 멀티플랫폼 매니페스트 리스트 생성
- **Docker Best Practices**: 멀티 스테이지 빌드, 레이어 캐싱 최적화, .dockerignore 사용
- **Constitution**: Workspace modularity 유지, 관찰 가능성 (빌드 로그)

---

## Revision History

| Date | Version | Changes |
|------|---------|---------|
| 2025-01-18 | 1.0.0 | 초기 계약 정의 |
