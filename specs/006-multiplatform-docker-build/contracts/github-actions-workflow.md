# GitHub Actions Workflow Contract

**Feature**: [spec.md](../spec.md)  
**Date**: 2025-01-18

## Purpose

이 문서는 멀티플랫폼 Docker 이미지를 자동으로 빌드하고 Docker Hub에 푸시하는 GitHub Actions 워크플로우의 동작, 입력, 출력을 정의한다.

---

## Workflow Metadata

**Name**: `Docker Multiplatform Build`  
**File Path**: `.github/workflows/docker-build.yml`  
**Trigger Events**:
- `push` to `main` branch
- `pull_request` to any branch

**Permissions**:
```yaml
permissions:
  contents: read
  packages: write
```

---

## Input Contract

### Trigger Events

| Event | Branches | Action |
|-------|----------|--------|
| `push` | `main` | 이미지 빌드 및 Docker Hub에 푸시 |
| `pull_request` | 모든 브랜치 | 이미지 빌드만 (푸시 안 함) |

### Required Secrets

| Secret Name | Type | Description | Example |
|-------------|------|-------------|---------|
| `DOCKERHUB_USERNAME` | String | Docker Hub 사용자명 | `johndoe` |
| `DOCKERHUB_TOKEN` | String | Docker Hub 액세스 토큰 (비밀번호 아님) | `dckr_pat_abc123...` |

**Token 생성 방법**:
1. Docker Hub → Account Settings → Security → New Access Token
2. 권한: Read, Write, Delete (이미지 푸시 필요)
3. 생성된 토큰을 GitHub Secrets에 저장

---

## Workflow Steps

### Step 1: Checkout

**Purpose**: 소스 코드 체크아웃

**Action**: `actions/checkout@v4`

**Contract**:
```yaml
- name: Checkout code
  uses: actions/checkout@v4
```

**Guarantees**:
- 현재 커밋의 전체 소스 코드를 러너에 다운로드
- `.dockerignore` 파일이 존재하면 빌드 컨텍스트에서 제외

---

### Step 2: Set up QEMU

**Purpose**: ARM 플랫폼 에뮬레이션 지원

**Action**: `docker/setup-qemu-action@v3`

**Contract**:
```yaml
- name: Set up QEMU
  uses: docker/setup-qemu-action@v3
```

**Guarantees**:
- ARM64 및 ARMv7 에뮬레이션 가능
- QEMU는 크로스 컴파일 시 보조적으로 사용 (주로 런타임 테스트)

---

### Step 3: Set up Docker Buildx

**Purpose**: Docker BuildKit 멀티플랫폼 빌더 설정

**Action**: `docker/setup-buildx-action@v3`

**Contract**:
```yaml
- name: Set up Docker Buildx
  uses: docker/setup-buildx-action@v3
```

**Guarantees**:
- 멀티플랫폼 빌드 지원
- BuildKit 캐시 백엔드 (GitHub Actions 캐시) 활성화
- 빌더 인스턴스: `default` (자동 생성)

---

### Step 4: Login to Docker Hub

**Purpose**: Docker Hub 인증

**Action**: `docker/login-action@v3`

**Contract**:
```yaml
- name: Login to Docker Hub
  uses: docker/login-action@v3
  with:
    username: ${{ secrets.DOCKERHUB_USERNAME }}
    password: ${{ secrets.DOCKERHUB_TOKEN }}
```

**Guarantees**:
- 인증 실패 시 워크플로우 중단 (exit 1)
- 토큰 만료 시 명확한 오류 메시지

**Error Handling**:
- `Invalid credentials` → DOCKERHUB_TOKEN 재생성 필요
- `API rate limit exceeded` → Docker Hub 플랜 업그레이드 또는 대기

---

### Step 5: Build and Push (Backend)

**Purpose**: Backend 이미지 빌드 및 푸시

**Action**: `docker/build-push-action@v6`

**Contract**:
```yaml
- name: Build and push backend
  uses: docker/build-push-action@v6
  with:
    context: .
    file: ./Dockerfile
    platforms: linux/amd64,linux/arm64,linux/arm/v7
    push: ${{ github.event_name == 'push' && github.ref == 'refs/heads/main' }}
    tags: |
      ${{ secrets.DOCKERHUB_USERNAME }}/realtime-svg-backend:latest
      ${{ secrets.DOCKERHUB_USERNAME }}/realtime-svg-backend:${{ github.sha }}
    build-args: |
      TARGET_BINARY=backend
      RUST_VERSION=1.86
    cache-from: type=gha
    cache-to: type=gha,mode=max
```

**Input Parameters**:
- `context`: 빌드 컨텍스트 디렉토리 (`.` = repository root)
- `file`: Dockerfile 경로
- `platforms`: 빌드할 플랫폼 목록 (쉼표 구분)
- `push`: `true`일 때만 레지스트리에 푸시
- `tags`: 이미지 태그 목록 (줄바꿈 구분)
- `build-args`: Dockerfile ARG 값
- `cache-from`: 캐시 소스 (GitHub Actions 캐시)
- `cache-to`: 캐시 대상 (`mode=max`는 모든 레이어 캐시)

**Conditional Push Logic**:
```yaml
push: ${{ github.event_name == 'push' && github.ref == 'refs/heads/main' }}
```
- **Push**: `push` 이벤트이고 `main` 브랜치일 때만
- **No Push**: PR 또는 다른 브랜치에서는 빌드만 수행 (검증용)

**Output**:
- 성공 시: Docker Hub에 3개 플랫폼 이미지 + 매니페스트 리스트 푸시
- 실패 시: 워크플로우 중단, 에러 로그 출력

---

### Step 6: Build and Push (Frontend) - Optional

**Purpose**: Frontend 이미지 빌드 및 푸시 (선택 사항)

**Contract**:
```yaml
- name: Build and push frontend
  uses: docker/build-push-action@v6
  with:
    context: .
    file: ./Dockerfile.frontend
    platforms: linux/amd64,linux/arm64,linux/arm/v7
    push: ${{ github.event_name == 'push' && github.ref == 'refs/heads/main' }}
    tags: |
      ${{ secrets.DOCKERHUB_USERNAME }}/realtime-svg-frontend:latest
      ${{ secrets.DOCKERHUB_USERNAME }}/realtime-svg-frontend:${{ github.sha }}
    build-args: |
      TARGET_BINARY=frontend
      RUST_VERSION=1.86
    cache-from: type=gha,scope=frontend
    cache-to: type=gha,mode=max,scope=frontend
```

**Cache Scope**:
- Backend: `type=gha` (기본 스코프)
- Frontend: `type=gha,scope=frontend` (독립 캐시)

이렇게 하면 backend와 frontend 빌드 캐시가 서로 간섭하지 않는다.

---

## Output Contract

### Success Criteria

**When Push Occurs** (`push` to `main`):
1. Docker Hub에 이미지가 푸시됨:
   - `username/realtime-svg-backend:latest`
   - `username/realtime-svg-backend:<commit-sha>`
2. 매니페스트 리스트 생성됨:
   ```bash
   docker manifest inspect username/realtime-svg-backend:latest
   ```
   출력에 3개 플랫폼 (amd64, arm64, arm/v7) 포함

3. 각 플랫폼별 이미지 크기 <100MB

**When No Push** (PR):
1. 이미지 빌드 성공 (검증만)
2. Docker Hub에 푸시 안 됨
3. 캐시만 GitHub Actions에 저장됨

---

### Logs and Artifacts

**Build Logs**:
- GitHub Actions UI에서 각 단계별 로그 확인
- Docker BuildKit 출력:
  ```
  [+] Building 120.5s (25/25) FINISHED
   => [linux/amd64 builder 1/5] RUN rustup target add x86_64-unknown-linux-gnu
   => [linux/arm64 builder 1/5] RUN rustup target add aarch64-unknown-linux-gnu
   => [linux/arm/v7 builder 1/5] RUN rustup target add armv7-unknown-linux-gnueabihf
  ```

**Cache Hit Rate**:
- 첫 실행: 캐시 미스 (5-8분 빌드)
- 이후 실행: 캐시 히트 (1-3분 빌드)
- Cargo.toml 변경 시: 부분 캐시 히트 (3-5분 빌드)

---

## Error Scenarios

| Error | Cause | Resolution |
|-------|-------|------------|
| `Invalid credentials` | DOCKERHUB_TOKEN 잘못됨 | GitHub Secrets 업데이트 |
| `denied: requested access to the resource is denied` | Docker Hub 권한 부족 | 토큰 권한 확인 (Write 필요) |
| `failed to solve: process "/bin/sh -c cargo build" did not complete successfully` | Rust 빌드 오류 | 로컬에서 `cargo build` 테스트 |
| `ERROR: Unsupported platform linux/riscv64` | 지원되지 않는 플랫폼 | `platforms` 목록 수정 |
| `cache export error` | GitHub Actions 캐시 크기 초과 (10GB) | 캐시 모드를 `min`으로 변경 또는 정리 |

---

## Performance Guarantees

| Scenario | Expected Duration | Cache Usage |
|----------|-------------------|-------------|
| 첫 실행 (캐시 없음) | 8-12분 | 0% |
| 소스 코드만 변경 | 2-4분 | ~70% (종속성 캐시 히트) |
| Cargo.toml 변경 | 5-8분 | ~30% (Planner 캐시 미스) |
| Dockerfile 변경 | 8-12분 | 0% (전체 캐시 무효화) |

---

## Workflow Validation

### Pre-Merge Checklist

워크플로우 병합 전 확인 사항:

- [ ] GitHub Secrets에 `DOCKERHUB_USERNAME` 설정됨
- [ ] GitHub Secrets에 `DOCKERHUB_TOKEN` 설정됨
- [ ] `Dockerfile` 파일이 루트에 존재
- [ ] `.dockerignore` 파일 생성 (선택 사항이지만 권장)
- [ ] PR에서 빌드 성공 확인 (푸시 안 됨)
- [ ] `main` 푸시 후 Docker Hub에 이미지 생성 확인

### Post-Deploy Verification

배포 후 검증:

```bash
# 1. 이미지 pull 테스트
docker pull username/realtime-svg-backend:latest

# 2. 매니페스트 확인
docker manifest inspect username/realtime-svg-backend:latest

# 3. ARM64 플랫폼에서 실행 테스트
docker run --rm --platform linux/arm64 username/realtime-svg-backend:latest --version

# 4. 이미지 크기 확인
docker images username/realtime-svg-backend:latest
```

---

## Security Considerations

### Secrets Management

- ❌ **절대 금지**: 워크플로우 파일에 토큰 하드코딩
- ✅ **권장**: GitHub Secrets 사용
- ✅ **권장**: Docker Hub 토큰 주기적 갱신 (90일마다)

### Image Scanning (Optional Enhancement)

워크플로우에 보안 스캔 추가 (선택 사항):

```yaml
- name: Run Trivy vulnerability scanner
  uses: aquasecurity/trivy-action@master
  with:
    image-ref: ${{ secrets.DOCKERHUB_USERNAME }}/realtime-svg-backend:latest
    format: 'sarif'
    output: 'trivy-results.sarif'

- name: Upload Trivy results to GitHub Security
  uses: github/codeql-action/upload-sarif@v2
  with:
    sarif_file: 'trivy-results.sarif'
```

---

## Compliance

이 워크플로우는 다음 표준을 준수한다:

- **GitHub Actions Best Practices**: 최소 권한 원칙, 시크릿 사용
- **Docker Best Practices**: 멀티 스테이지 빌드, 레이어 캐싱
- **Constitution**: 관찰 가능성 (빌드 로그), 단순성 (단일 워크플로우)

---

## Revision History

| Date | Version | Changes |
|------|---------|---------|
| 2025-01-18 | 1.0.0 | 초기 계약 정의 |
