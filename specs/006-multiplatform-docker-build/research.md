# Research: 멀티플랫폼 Docker 빌드 설정

**Date**: 2025-01-18  
**Feature**: [spec.md](./spec.md)  
**Plan**: [plan.md](./plan.md)

## Purpose

이 문서는 Rust 워크스페이스 프로젝트의 멀티플랫폼 Docker 빌드 구현을 위한 기술 조사 결과를 정리한다. 주요 의사결정 사항과 근거를 문서화하여 구현 단계에서 참조할 수 있도록 한다.

---

## 1. Dockerfile 구조: Backend vs Frontend 분리 전략

### Decision
별도의 Dockerfile 사용 (`Dockerfile`, `Dockerfile.frontend`)

### Rationale
- **명확한 관심사 분리**: Backend와 frontend는 빌드 프로세스, 종속성, 런타임 요구사항이 다르다.
- **유지보수성 향상**: 각 Dockerfile이 단일 책임을 가져 디버깅과 수정이 용이하다.
- **CI/CD 간소화**: GitHub Actions에서 각 이미지를 독립적으로 빌드하고 배포할 수 있다.
- **빌드 캐시 효율성**: 한 쪽의 변경이 다른 쪽의 캐시를 무효화하지 않는다.

### Alternatives Considered

**대안 1: 단일 Dockerfile + 빌드 아규먼트**
```dockerfile
ARG TARGET=backend
RUN cargo build --release --bin ${TARGET}
```
- ❌ **거부 이유**: 조건부 로직이 복잡해지며, 각 타겟의 특수한 요구사항을 처리하기 어렵다. 예를 들어 frontend는 trunk 빌드가 필요할 수 있으나 backend는 cargo만 필요하다.

**대안 2: 공유 Dockerfile (모든 바이너리 포함)**
```dockerfile
RUN cargo build --release --workspace
```
- ❌ **거부 이유**: 불필요한 바이너리가 최종 이미지에 포함되어 크기가 증가하고, 보안 공격 표면이 넓어진다.

### Implementation Notes

```dockerfile
# Dockerfile (backend용)
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin backend
COPY . .
RUN cargo build --release --bin backend

FROM debian:bookworm-slim AS runtime
COPY --from=builder /app/target/release/backend /usr/local/bin
ENTRYPOINT ["/usr/local/bin/backend"]
```

**구조 선택**: Backend와 frontend에 각각 별도 Dockerfile을 생성하며, 공통 빌드 단계는 유사한 구조를 유지한다.

---

## 2. Cargo 종속성 캐싱 전략

### Decision
`cargo-chef`를 사용한 종속성 캐싱

### Rationale
- **빌드 시간 대폭 단축**: 종속성 변경이 없을 때 5-10배 빌드 시간 절감 (실제 프로젝트에서 ~10분 → ~2분)
- **워크스페이스 자동 감지**: 멀티 크레이트 구조를 자동으로 처리한다.
- **파일 이동 대응**: 소스 파일 구조 변경 시에도 캐시가 깨지지 않는다.
- **유지보수 용이**: cargo-chef가 recipe.json을 자동 생성하므로 수동 관리가 불필요하다.

### Alternatives Considered

**대안 1: 더미 빌드 접근법**
```dockerfile
RUN mkdir src && echo 'fn main() {println!("dummy")}' > src/main.rs
RUN cargo build --release
RUN rm -rf src
COPY src ./src
RUN cargo build --release
```
- ❌ **거부 이유**: 
  - 파일 구조 변경 시 수동 업데이트 필요
  - lib.rs와 main.rs 혼재 시 처리 복잡
  - 워크스페이스 구조에서 오류 발생 가능

**대안 2: 캐싱 없음**
```dockerfile
COPY . .
RUN cargo build --release
```
- ❌ **거부 이유**: 종속성이 변경되지 않아도 매번 전체 재컴파일로 빌드 시간이 10분 이상 소요된다.

### Implementation Notes

```dockerfile
# 3단계 캐싱 접근법
FROM lukemathwalker/cargo-chef:latest-rust-1.86 AS chef
WORKDIR /app

# 1단계: Recipe 생성 (종속성 그래프 추출)
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# 2단계: 종속성 빌드 (캐시 활용)
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin backend

# 3단계: 애플리케이션 빌드
COPY . .
RUN cargo build --release --bin backend
```

**효과**: Cargo.toml 변경 없이 소스 코드만 수정 시 2단계는 캐시에서 로드되어 빌드 시간이 50% 이상 단축된다.

---

## 3. 크로스 컴파일 링커 설정

### Decision
플랫폼별 `CARGO_TARGET_*_LINKER` 환경 변수 설정 및 크로스 컴파일러 설치

### Rationale
- **빌드 속도**: QEMU 에뮬레이션 대비 크로스 컴파일이 3-5배 빠르다.
- **확실한 바이너리 타겟**: 올바른 아키텍처용 바이너리 생성을 보장한다.
- **안정성**: 에뮬레이션 오류 및 타임아웃 문제를 회피한다.

### Alternatives Considered

**대안 1: QEMU 에뮬레이션만 사용**
```dockerfile
FROM --platform=$TARGETPLATFORM rust:1.86
RUN cargo build --release
```
- ❌ **거부 이유**: 
  - ARM64 에뮬레이션에서 Rust 컴파일 시 10분 이상 소요
  - 간헐적인 QEMU 크래시 및 타임아웃 발생
  - CPU 집약적 작업에서 매우 느림

**대안 2: 네이티브 빌더 (플랫폼별 러너)**
```yaml
# GitHub Actions
strategy:
  matrix:
    platform: [ubuntu-latest, ubuntu-arm64, ubuntu-armv7]
```
- ❌ **거부 이유**: 
  - ARM 러너는 비용이 높고 GitHub Actions에서 제한적으로 지원
  - 인프라 오버헤드 증가
  - 빌드 병렬화 어려움

### Implementation Notes

```dockerfile
FROM --platform=$BUILDPLATFORM rust:1.86-bookworm AS builder

ARG TARGETPLATFORM

# 1. 플랫폼에 따른 Rust 타겟 결정
RUN case "$TARGETPLATFORM" in \
      "linux/amd64") echo x86_64-unknown-linux-gnu > /rust_target.txt ;; \
      "linux/arm64") echo aarch64-unknown-linux-gnu > /rust_target.txt ;; \
      "linux/arm/v7") echo armv7-unknown-linux-gnueabihf > /rust_target.txt ;; \
      *) echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
    esac

# 2. Rust 타겟 추가
RUN export RUST_TARGET=$(cat /rust_target.txt) && \
    rustup target add $RUST_TARGET

# 3. 크로스 컴파일러 설치
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc-aarch64-linux-gnu \
    gcc-arm-linux-gnueabihf \
    gcc-x86-64-linux-gnu \
    libc6-dev-arm64-cross \
    libc6-dev-armhf-cross \
    && rm -rf /var/lib/apt/lists/*

# 4. 링커 설정 및 빌드
RUN export RUST_TARGET=$(cat /rust_target.txt) && \
    case "$RUST_TARGET" in \
      "aarch64-unknown-linux-gnu") \
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc ;; \
      "armv7-unknown-linux-gnueabihf") \
        export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc ;; \
    esac && \
    cargo build --release --target $RUST_TARGET --bin backend
```

**일반적인 오류 및 해결법**:
- `error: linker 'cc' not found` → 크로스 컴파일러 패키지 설치 필요
- `cannot find -lgcc_s` → libc 개발 패키지 (`libc6-dev-*-cross`) 설치 필요
- 잘못된 아키텍처 바이너리 → `file` 명령으로 바이너리 검증

---

## 4. GitHub Actions 멀티플랫폼 빌드 설정

### Decision
`docker/build-push-action`과 GitHub Actions 캐시 백엔드 사용

### Rationale
- **공식 지원**: Docker 공식 GitHub Action으로 안정적이고 문서화가 잘 되어 있다.
- **네이티브 멀티플랫폼**: BuildKit의 멀티플랫폼 빌드를 완벽히 지원한다.
- **효율적 캐싱**: GitHub Actions 캐시 백엔드는 추가 인프라 없이 빌드 캐시를 저장한다.
- **간단한 시크릿 관리**: GitHub Secrets와 자연스럽게 통합된다.

### Alternatives Considered

**대안 1: 다중 러너 전략 (플랫폼별 병렬 빌드)**
```yaml
strategy:
  matrix:
    platform: [amd64, arm64, armv7]
runs-on: ${{ matrix.platform }}-runner
```
- ❌ **거부 이유**: 
  - ARM 러너 비용이 높고 가용성이 제한적
  - 이미지 매니페스트 병합 과정이 복잡
  - CI 분당 비용 증가

**대안 2: 로컬 캐시 (cache-from/to: type=local)**
```yaml
cache-from: type=local,src=/tmp/.buildx-cache
cache-to: type=local,dest=/tmp/.buildx-cache,mode=max
```
- ❌ **거부 이유**: 
  - 10GB 캐시 크기 제한으로 대형 프로젝트에서 부족
  - 수동 캐시 정리 필요
  - 팀원 간 캐시 공유 불가

### Implementation Notes

```yaml
name: Docker Multiplatform Build

on:
  push:
    branches: [main]
  pull_request:

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      
      - name: Set up QEMU
        uses: docker/setup-qemu-action@v3
      
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3
      
      - name: Login to Docker Hub
        uses: docker/login-action@v3
        with:
          username: ${{ secrets.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}
      
      - name: Build and push backend
        uses: docker/build-push-action@v6
        with:
          context: .
          file: ./Dockerfile
          platforms: linux/amd64,linux/arm64,linux/arm/v7
          push: ${{ github.event_name != 'pull_request' }}
          tags: |
            ${{ secrets.DOCKERHUB_USERNAME }}/realtime-svg-backend:latest
            ${{ secrets.DOCKERHUB_USERNAME }}/realtime-svg-backend:${{ github.sha }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

**캐시 백엔드 비교**:
- **`type=gha`** (선택됨): GitHub Actions 캐시, 10GB 제한, 팀 공유 가능
- **`type=registry`**: 레지스트리 저장, 무제한 크기, 추가 스토리지 비용
- **`type=inline`**: 이미지 내 캐시, `min` 모드만 지원, 크기 증가

**시크릿 관리**:
```yaml
# GitHub Secrets 설정 필요
# DOCKERHUB_USERNAME: Docker Hub 사용자명
# DOCKERHUB_TOKEN: Docker Hub 액세스 토큰 (비밀번호 아님)
```

---

## 5. 런타임 이미지 최적화

### Decision
`debian:bookworm-slim` 베이스 이미지 사용 (glibc 기반)

### Rationale
- **광범위한 호환성**: 대부분의 Rust crate가 glibc를 기대하며, 네이티브 라이브러리 종속성이 있는 crate도 지원한다.
- **안정성**: OpenSSL, libpq 등 C 라이브러리 의존성이 있는 경우 musl 대비 안정적이다.
- **디버깅 용이성**: 쉘 및 기본 도구가 포함되어 있어 문제 해결이 쉽다.
- **적정 크기**: ~80MB로 scratch 대비 크지만 실용적이다.

### Alternatives Considered

**대안 1: Alpine Linux (musl 기반)**
```dockerfile
FROM alpine:latest AS runtime
RUN apk --no-cache add ca-certificates
```
- ❌ **거부 이유**: 
  - musl 타겟으로 정적 링킹 필요 (빌드 복잡도 증가)
  - OpenSSL, PostgreSQL 클라이언트 등 네이티브 라이브러리 crate에서 호환성 문제
  - 성능 이슈 (musl의 메모리 할당이 glibc보다 느림)
  - 이미지 크기 절감 효과가 미미 (~5MB vs ~80MB, 애플리케이션 바이너리가 대부분 차지)

**대안 2: Distroless 이미지**
```dockerfile
FROM gcr.io/distroless/cc-debian12
```
- ❌ **거부 이유**: 
  - 쉘 및 디버깅 도구 부재로 문제 해결 어려움
  - tini 같은 프로세스 관리 도구 추가 설치 불가
  - 보안 이점이 크지 않음 (debian-slim도 최소 패키지만 포함)

**대안 3: Scratch (빈 이미지)**
```dockerfile
FROM scratch
```
- ❌ **거부 이유**: 
  - 완전한 정적 링킹 필요 (musl + 모든 C 라이브러리 정적 포함)
  - CA 인증서 번들 수동 복사 필요
  - 디버깅 불가능 (exec 명령 사용 불가)

### Implementation Notes

```dockerfile
# 최종 런타임 이미지
FROM debian:bookworm-slim AS runtime

# 필수 런타임 종속성 설치
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    tini \
    && rm -rf /var/lib/apt/lists/*

# 빌더 스테이지에서 바이너리 복사
COPY --from=builder /app/target/release/backend /usr/local/bin/backend

# tini를 PID 1로 사용하여 시그널 처리 및 좀비 프로세스 방지
ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/backend"]
```

**필수 런타임 종속성**:
- **`ca-certificates`**: HTTPS/TLS 연결을 위한 루트 인증서 번들 (Redis TLS, 외부 API 호출)
- **`tini`**: PID 1 프로세스 관리, 시그널 전달 및 좀비 프로세스 정리
- **`libgcc1`** (자동 포함): 스택 언와인딩 및 예외 처리

**바이너리 검증**:
```bash
# 바이너리 아키텍처 확인
docker run --rm --entrypoint file realtime-svg-backend /usr/local/bin/backend
# 출력 예시: ELF 64-bit LSB executable, ARM aarch64, version 1 (SYSV)

# 동적 링킹 종속성 확인
docker run --rm --entrypoint ldd realtime-svg-backend /usr/local/bin/backend
# 출력 예시:
#   linux-vdso.so.1
#   libc.so.6 => /lib/aarch64-linux-gnu/libc.so.6
#   libgcc_s.so.1 => /lib/aarch64-linux-gnu/libgcc_s.so.1
```

**이미지 크기 비교** (backend 바이너리 ~20MB 가정):
- `debian:bookworm-slim` + backend: **~100MB** (선택됨)
- `alpine` + musl static backend: **~25MB** (호환성 문제)
- `scratch` + static backend: **~20MB** (디버깅 불가)

---

## Summary

이 조사를 통해 다음 기술 결정을 확정했다:

1. **Dockerfile 구조**: Backend와 frontend를 위한 별도 Dockerfile 사용
2. **종속성 캐싱**: `cargo-chef`를 사용한 3단계 빌드 접근법
3. **크로스 컴파일**: 플랫폼별 링커 설정 및 크로스 컴파일러 도구 사용
4. **CI/CD**: GitHub Actions의 `docker/build-push-action`과 GHA 캐시 백엔드
5. **런타임 이미지**: `debian:bookworm-slim` 베이스에 `tini` 및 `ca-certificates` 포함

모든 결정은 빌드 시간 최소화, 이미지 크기 최적화, 호환성 보장, 유지보수성 향상을 목표로 한다.
