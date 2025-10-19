# Container Image Contract

**Feature**: 007-k8s-deployment-packaging  
**Version**: 1.0.0

## Purpose

배포 패키지가 사용하는 컨테이너 이미지에 대한 요구사항을 정의합니다.

## Unified Application Image Contract

### Image Name

- **Repository**: `ghcr.io/egoavara/realtime-svg` (기본값, 사용자 설정 가능)
- **Tag**: `v0.1.4` (기본값, 사용자 설정 가능)
- **Full Reference**: `ghcr.io/egoavara/realtime-svg:v0.1.4`

**참고**: 이 이미지는 frontend와 backend를 모두 포함하는 통합 이미지입니다.

### Required Capabilities

1. **Port Exposure**
   - 포트 8080에서 HTTP 서버 리스닝
   
2. **Environment Variables**
   - `REDIS_HOST`: Redis 서버 호스트 (필수)
   - `REDIS_PORT`: Redis 서버 포트 (기본: 6379)
   - `REDIS_PASSWORD`: Redis 비밀번호 (선택)
   - `LOG_LEVEL`: 로그 레벨 (기본: info)
   - `BACKEND_URL`: 백엔드 URL (ConfigMap 제공)

3. **Health Endpoints**
   - `GET /health`: Liveness probe 엔드포인트
     - 성공: HTTP 200
     - 실패: HTTP 503
   - `GET /ready`: Readiness probe 엔드포인트
     - 성공: HTTP 200
     - 실패: HTTP 503

4. **Runtime Requirements**
   - 최소 CPU: 100m
   - 최소 메모리: 128Mi
   - 권장 CPU 제한: 500m
   - 권장 메모리 제한: 512Mi

5. **Signal Handling**
   - `SIGTERM` 수신 시 graceful shutdown
   - Shutdown timeout: 30초

### Container Architecture

이 통합 이미지는 이미 빌드되어 GitHub Container Registry에 게시되어 있습니다. Dockerfile은 참고용으로만 제공됩니다.

**참고**: 실제 이미지 빌드 프로세스는 프로젝트의 CI/CD 파이프라인에서 관리되며, 배포 패키지는 기존 이미지를 사용합니다.

---

## Redis Image Contract (Optional)

### Image Name

- **Repository**: `redis` (공식 이미지)
- **Tag**: `7-alpine` (권장)
- **Full Reference**: `redis:7-alpine`

### Configuration

1. **Port Exposure**
   - 포트 6379에서 Redis 프로토콜 리스닝

2. **Command Override**
   - 인메모리 모드: `redis-server --save "" --appendonly no`
   - 영속성 비활성화로 데이터 유실 가능 (개발/테스트 용도)

3. **Environment Variables**
   - (선택) `REDIS_PASSWORD`: 비밀번호 설정

4. **Runtime Requirements**
   - 최소 CPU: 100m
   - 최소 메모리: 128Mi
   - 권장 CPU 제한: 200m
   - 권장 메모리 제한: 256Mi

5. **Data Persistence**
   - **경고**: 클러스터 내 Redis는 PVC 없이 인메모리로 실행
   - Pod 재시작 시 모든 데이터 유실
   - 프로덕션에서는 외부 관리형 Redis 사용 필수

---

## Image Registry Requirements

### Public Registry (기본)

- Docker Hub, ghcr.io 등 공개 레지스트리 사용
- 인증 불필요

### Private Registry (선택)

배포 패키지는 프라이빗 레지스트리 지원을 위해 다음 설정을 허용:

```yaml
# Helm values.yaml
image:
  backend:
    repository: myregistry.azurecr.io/realtime-svg-backend
    pullPolicy: Always
  pullSecrets:
    - name: registry-secret
```

사용자는 수동으로 `imagePullSecrets`를 생성해야 함:

```bash
kubectl create secret docker-registry registry-secret \
  --docker-server=myregistry.azurecr.io \
  --docker-username=myuser \
  --docker-password=mypassword
```

---

## Multi-Architecture Support

### Current Status

- **amd64**: 지원 (기본)
- **arm64**: 선택적 지원 (사용자가 멀티 아키텍처 이미지 빌드 시)

### Multi-arch Build Example

```bash
docker buildx build --platform linux/amd64,linux/arm64 \
  -t realtime-svg/backend:latest \
  --push \
  ./crates/backend
```

배포 패키지는 아키텍처를 명시하지 않으며, Kubernetes가 노드 아키텍처에 맞는 이미지 선택합니다.

---

## Version Compatibility

- Kubernetes 1.24+와 호환되는 컨테이너 런타임 필요
- OCI 표준 준수 이미지
- 최대 이미지 크기 권장: 500MB 이하 (빠른 pull을 위해)

---

## Security Best Practices

1. **Non-root User**
   - 컨테이너는 root가 아닌 사용자로 실행 권장
   - `USER 1000` Dockerfile 지시어 사용

2. **Read-only Filesystem**
   - 가능하면 읽기 전용 루트 파일시스템 사용
   - 쓰기 필요 시 emptyDir 볼륨 마운트

3. **Vulnerability Scanning**
   - CI/CD에서 이미지 취약점 스캔 권장 (Trivy, Snyk 등)

4. **Image Signing**
   - (선택) Sigstore/Cosign으로 이미지 서명 및 검증
