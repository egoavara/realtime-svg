# Quickstart: 멀티플랫폼 Docker 빌드

**Feature**: [spec.md](./spec.md)  
**Plan**: [plan.md](./plan.md)  
**Date**: 2025-01-18

## Purpose

이 가이드는 개발자가 멀티플랫폼 Docker 이미지를 빠르게 빌드하고 배포할 수 있도록 단계별 지침을 제공한다.

---

## Prerequisites

시작하기 전에 다음 도구가 설치되어 있는지 확인하세요:

### Required

- **Docker Desktop** (버전 20.10 이상)
  - [macOS](https://docs.docker.com/desktop/install/mac-install/)
  - [Windows](https://docs.docker.com/desktop/install/windows-install/)
  - [Linux](https://docs.docker.com/desktop/install/linux-install/)

- **Docker Buildx** (Docker Desktop에 기본 포함)
  ```bash
  docker buildx version
  # 출력 예시: github.com/docker/buildx v0.12.0
  ```

### Optional (CI/CD용)

- **GitHub Account** (GitHub Actions 사용 시)
- **Docker Hub Account** (이미지 푸시 시)
  - [Docker Hub 가입](https://hub.docker.com/signup)

---

## Step 1: 프로젝트 준비

### 1.1. 코드 체크아웃

```bash
git clone https://github.com/egoavara/realtime-svg.git
cd realtime-svg
git checkout 006-multiplatform-docker-build
```

### 1.2. .dockerignore 생성 (선택 사항이지만 권장)

빌드 컨텍스트에서 불필요한 파일을 제외하여 빌드 속도를 향상시킵니다:

```bash
cat > .dockerignore <<EOF
target/
.git/
.github/
specs/
*.md
.env*
.vscode/
.idea/
EOF
```

---

## Step 2: 로컬에서 단일 플랫폼 빌드 (테스트용)

멀티플랫폼 빌드 전에 로컬 플랫폼용으로 빌드하여 Dockerfile을 검증합니다.

### 2.1. Backend 빌드

```bash
docker build \
  --build-arg TARGET_BINARY=backend \
  --tag realtime-svg-backend:test \
  --file Dockerfile \
  .
```

**예상 결과**:
- 빌드 시간: 3-5분 (종속성 캐시 없을 때)
- 이미지 크기: ~100MB 이하

### 2.2. 빌드된 이미지 실행

```bash
docker run --rm realtime-svg-backend:test --help
```

**예상 출력**:
```
Usage: backend [OPTIONS]

Options:
  --host <HOST>        Server host [default: 127.0.0.1]
  --port <PORT>        Server port [default: 3000]
  --config <CONFIG>    Config file path
  -h, --help           Print help
```

### 2.3. 이미지 크기 확인

```bash
docker images realtime-svg-backend:test
```

**예상 출력**:
```
REPOSITORY               TAG     IMAGE ID       CREATED          SIZE
realtime-svg-backend     test    abc123def456   2 minutes ago    95MB
```

---

## Step 3: 멀티플랫폼 빌드 (로컬)

### 3.1. Buildx 빌더 생성 (첫 실행 시만)

```bash
docker buildx create --name multiplatform-builder --use
docker buildx inspect --bootstrap
```

**예상 출력**:
```
[+] Building 2.5s (1/1) FINISHED
 => [internal] booting buildkit
Name:   multiplatform-builder
Driver: docker-container

Platforms: linux/amd64, linux/arm64, linux/arm/v7, ...
```

### 3.2. 멀티플랫폼 빌드 (로컬 레지스트리 없이)

```bash
docker buildx build \
  --platform linux/amd64,linux/arm64,linux/arm/v7 \
  --build-arg TARGET_BINARY=backend \
  --tag realtime-svg-backend:multiplatform \
  --file Dockerfile \
  --load \
  .
```

**참고**: `--load` 플래그는 단일 플랫폼만 로컬에 로드합니다 (현재 플랫폼). 모든 플랫폼을 테스트하려면 레지스트리에 푸시해야 합니다.

**예상 결과**:
- 빌드 시간: 5-8분 (종속성 캐시 없을 때)
- 3개 플랫폼 빌드 성공

---

## Step 4: Docker Hub에 푸시

### 4.1. Docker Hub 로그인

```bash
docker login
# Username: your-dockerhub-username
# Password: your-dockerhub-token (비밀번호 아님)
```

**Docker Hub 토큰 생성**:
1. [Docker Hub](https://hub.docker.com) → Account Settings → Security
2. "New Access Token" 클릭
3. 권한: Read, Write, Delete 선택
4. 토큰 복사 (다시 볼 수 없음)

### 4.2. 멀티플랫폼 빌드 및 푸시

```bash
docker buildx build \
  --platform linux/amd64,linux/arm64,linux/arm/v7 \
  --build-arg TARGET_BINARY=backend \
  --tag your-username/realtime-svg-backend:latest \
  --tag your-username/realtime-svg-backend:v1.0.0 \
  --file Dockerfile \
  --push \
  .
```

**예상 결과**:
```
[+] Building 320.5s (75/75) FINISHED
 => [linux/amd64 runtime 1/3] FROM debian:bookworm-slim
 => [linux/arm64 runtime 1/3] FROM debian:bookworm-slim
 => [linux/arm/v7 runtime 1/3] FROM debian:bookworm-slim
 => => pushing manifest for docker.io/your-username/realtime-svg-backend:latest
```

### 4.3. 매니페스트 확인

```bash
docker manifest inspect your-username/realtime-svg-backend:latest
```

**예상 출력**:
```json
{
  "manifests": [
    {
      "platform": {
        "architecture": "amd64",
        "os": "linux"
      }
    },
    {
      "platform": {
        "architecture": "arm64",
        "os": "linux"
      }
    },
    {
      "platform": {
        "architecture": "arm",
        "os": "linux",
        "variant": "v7"
      }
    }
  ]
}
```

---

## Step 5: 플랫폼별 이미지 테스트

### 5.1. AMD64에서 실행

```bash
docker run --rm --platform linux/amd64 \
  your-username/realtime-svg-backend:latest \
  --help
```

### 5.2. ARM64에서 실행 (에뮬레이션)

```bash
docker run --rm --platform linux/arm64 \
  your-username/realtime-svg-backend:latest \
  --help
```

**참고**: 로컬 머신이 AMD64일 경우 QEMU 에뮬레이션을 통해 ARM 이미지를 실행합니다 (느릴 수 있음).

### 5.3. 바이너리 아키텍처 확인

```bash
docker run --rm --platform linux/amd64 \
  --entrypoint file \
  your-username/realtime-svg-backend:latest \
  /usr/local/bin/backend
```

**예상 출력**:
```
/usr/local/bin/backend: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), dynamically linked
```

ARM64에서:
```bash
docker run --rm --platform linux/arm64 \
  --entrypoint file \
  your-username/realtime-svg-backend:latest \
  /usr/local/bin/backend
```

**예상 출력**:
```
/usr/local/bin/backend: ELF 64-bit LSB executable, ARM aarch64, version 1 (SYSV), dynamically linked
```

---

## Step 6: GitHub Actions CI/CD 설정

### 6.1. GitHub Secrets 설정

1. GitHub Repository → Settings → Secrets and variables → Actions
2. "New repository secret" 클릭
3. 다음 시크릿 추가:
   - `DOCKERHUB_USERNAME`: Docker Hub 사용자명
   - `DOCKERHUB_TOKEN`: Docker Hub 액세스 토큰

### 6.2. 워크플로우 파일 생성

`.github/workflows/docker-build.yml` 파일이 이미 생성되어 있는지 확인:

```bash
cat .github/workflows/docker-build.yml
```

없다면 [contracts/github-actions-workflow.md](./contracts/github-actions-workflow.md)를 참조하여 생성하세요.

### 6.3. 워크플로우 트리거

```bash
git add .github/workflows/docker-build.yml Dockerfile .dockerignore
git commit -m "feat: add multiplatform Docker build workflow"
git push origin 006-multiplatform-docker-build
```

GitHub Actions → Workflows → "Docker Multiplatform Build"에서 빌드 진행 상황을 확인하세요.

**예상 결과**:
- PR 생성 시: 빌드만 수행 (푸시 안 됨)
- `main` 병합 시: 빌드 및 Docker Hub에 푸시

---

## Step 7: 프로덕션 배포

### 7.1. ARM64 서버에서 이미지 pull

```bash
# 예: Raspberry Pi 또는 AWS Graviton 인스턴스
docker pull your-username/realtime-svg-backend:latest
```

**자동 플랫폼 선택**:
Docker는 현재 플랫폼에 맞는 이미지를 자동으로 선택합니다. ARM64 서버에서는 ARM64 이미지를 다운로드합니다.

### 7.2. 컨테이너 실행

```bash
docker run -d \
  --name realtime-svg-backend \
  --restart unless-stopped \
  -p 3000:3000 \
  -e REDIS_URL=redis://redis-server:6379 \
  -e LOG_LEVEL=info \
  your-username/realtime-svg-backend:latest
```

### 7.3. 헬스체크

```bash
curl http://localhost:3000/
# 또는
docker logs realtime-svg-backend
```

---

## Troubleshooting

### 문제 1: `docker buildx` 명령을 찾을 수 없음

**원인**: Docker Buildx가 설치되지 않음

**해결**:
```bash
# Docker Desktop 재설치 또는
brew install docker-buildx-plugin  # macOS
```

---

### 문제 2: `denied: requested access to the resource is denied`

**원인**: Docker Hub 인증 실패 또는 권한 부족

**해결**:
1. Docker Hub 로그인 확인:
   ```bash
   docker logout
   docker login
   ```
2. 토큰 권한 확인 (Write 권한 필요)
3. 레포지토리명이 정확한지 확인 (`username/repo:tag`)

---

### 문제 3: `ERROR: Unsupported platform linux/riscv64`

**원인**: Dockerfile에서 지원하지 않는 플랫폼 지정

**해결**:
지원되는 플랫폼만 사용:
```bash
--platform linux/amd64,linux/arm64,linux/arm/v7
```

---

### 문제 4: 빌드가 매우 느림 (10분 이상)

**원인**: 종속성 캐시가 작동하지 않음

**해결**:
1. cargo-chef가 Dockerfile에 포함되어 있는지 확인
2. `.dockerignore`에 `target/` 디렉토리가 포함되어 있는지 확인
3. Docker BuildKit 캐시 활성화:
   ```bash
   export DOCKER_BUILDKIT=1
   ```

---

### 문제 5: ARM 이미지가 실행되지 않음

**원인**: 바이너리가 잘못된 아키텍처용으로 컴파일됨

**해결**:
1. 바이너리 아키텍처 확인:
   ```bash
   docker run --rm --entrypoint file \
     your-username/realtime-svg-backend:latest \
     /usr/local/bin/backend
   ```
2. 크로스 컴파일러 설정 확인 (Dockerfile의 `CARGO_TARGET_*_LINKER`)

---

## Performance Tips

### 1. 캐시 활용

GitHub Actions에서 캐시를 최대한 활용하려면:
```yaml
cache-from: type=gha
cache-to: type=gha,mode=max  # 모든 레이어 캐시
```

### 2. 병렬 빌드

여러 이미지 (backend, frontend)를 빌드할 때 병렬로 실행:
```bash
# Terminal 1
docker buildx build ... --tag backend:latest &

# Terminal 2
docker buildx build ... --tag frontend:latest &

wait  # 모든 빌드 완료 대기
```

### 3. 로컬 빌드 캐시

로컬에서 빌드 시 BuildKit 캐시를 활성화:
```bash
export DOCKER_BUILDKIT=1
docker build --cache-from type=local,src=/tmp/docker-cache ...
```

---

## Next Steps

- **[tasks.md](./tasks.md)**: 상세 구현 태스크 확인 (`/speckit.tasks` 명령으로 생성)
- **[contracts/](./contracts/)**: Dockerfile 및 GitHub Actions 계약 문서
- **[data-model.md](./data-model.md)**: 빌드 설정 데이터 모델

---

## Summary

이 가이드를 완료하면:
- ✅ 로컬에서 멀티플랫폼 Docker 이미지를 빌드할 수 있습니다.
- ✅ Docker Hub에 이미지를 푸시하고 다른 플랫폼에서 테스트할 수 있습니다.
- ✅ GitHub Actions로 자동화된 CI/CD 파이프라인을 설정할 수 있습니다.
- ✅ 프로덕션 환경에서 멀티플랫폼 이미지를 배포할 수 있습니다.

**예상 전체 소요 시간**: 30-60분 (처음 설정 시)
