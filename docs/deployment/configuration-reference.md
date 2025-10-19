# Configuration Reference

모든 배포 방법 (Helm, kubectl, Terraform, Pulumi)에서 사용 가능한 설정 매개변수 레퍼런스입니다.

## 공통 매개변수 구조

모든 배포 도구는 동일한 매개변수 이름과 구조를 사용합니다.

### Namespace

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `namespace` | string | `default` | Kubernetes 네임스페이스 |

### Image

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `image.repository` | string | `ghcr.io/egoavara/realtime-svg` | 컨테이너 이미지 저장소 |
| `image.tag` | string | `v0.1.4` | 이미지 태그 |
| `image.pullPolicy` | string | `IfNotPresent` | 이미지 풀 정책 (IfNotPresent, Always, Never) |

### Replicas

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `replicas` | integer | `2` | Pod 복제본 수 |

### Resources

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `resources.requests.cpu` | string | `100m` | CPU 요청량 |
| `resources.requests.memory` | string | `128Mi` | 메모리 요청량 |
| `resources.limits.cpu` | string | `500m` | CPU 제한 |
| `resources.limits.memory` | string | `512Mi` | 메모리 제한 |

### Redis

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `redis.enabled` | boolean | `true` | 클러스터 내 Redis 배포 여부 |
| `redis.password` | string | `""` | Redis 비밀번호 (enabled=true일 때, 선택) |
| `redis.external.url` | string | `""` | 외부 Redis URL (enabled=false일 때 필수, 예: redis://host:6379/) |
| `redis.resources.requests.cpu` | string | `100m` | Redis CPU 요청량 |
| `redis.resources.requests.memory` | string | `128Mi` | Redis 메모리 요청량 |
| `redis.resources.limits.cpu` | string | `200m` | Redis CPU 제한 |
| `redis.resources.limits.memory` | string | `256Mi` | Redis 메모리 제한 |

### Ingress

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `ingress.enabled` | boolean | `true` | Ingress 생성 여부 |
| `ingress.host` | string | `realtime-svg.example.com` | 호스트명 |
| `ingress.path` | string | `/` | 경로 |
| `ingress.pathType` | string | `Prefix` | 경로 타입 (Prefix, Exact) |
| `ingress.annotations` | object | `{}` | 커스텀 annotations |
| `ingress.tls.enabled` | boolean | `false` | TLS 활성화 |
| `ingress.tls.secretName` | string | `""` | TLS 시크릿 이름 |

### Service

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `service.type` | string | `ClusterIP` | 서비스 타입 (ClusterIP, NodePort, LoadBalancer) |
| `service.port` | integer | `80` | 서비스 포트 |

### Config

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `config.logLevel` | string | `info` | 로그 레벨 (info, debug, warn, error) |
| `config.port` | integer | `8080` | 애플리케이션 포트 |

**환경 변수:**
- `REDIS_URL`: Redis 연결 URL (자동 생성됨, redis.enabled 및 redis.external.url 기반)
- `LOG_LEVEL`: 로그 레벨
- `PORT`: 애플리케이션 포트
- `HOST`: 바인딩 주소 (고정값: 0.0.0.0)

### Secrets

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `secrets.apiKey` | string | `""` | API 키 (선택, base64 인코딩) |

## 배포 도구별 매핑

### Helm

```yaml
# values.yaml
namespace: production
image:
  repository: ghcr.io/egoavara/realtime-svg
  tag: v0.1.5
replicas: 3
redis:
  enabled: false
  external:
    url: redis://redis.external.svc:6379/
```

### Terraform

```hcl
# terraform.tfvars
namespace = "production"
image_repository = "ghcr.io/egoavara/realtime-svg"
image_tag = "v0.1.5"
replicas = 3
redis_enabled = false
redis_external_url = "redis://redis.external.svc:6379/"
```

### Pulumi

```bash
# CLI
pulumi config set namespace production
pulumi config set image:repository ghcr.io/egoavara/realtime-svg
pulumi config set image:tag v0.1.5
pulumi config set replicas 3
pulumi config set redis:enabled false
pulumi config set redis:external:url redis://redis.external.svc:6379/
```

### kubectl + Kustomize

```yaml
# kustomization.yaml
namespace: production
images:
  - name: realtime-svg
    newName: ghcr.io/egoavara/realtime-svg
    newTag: v0.1.5
replicas:
  - name: realtime-svg
    count: 3
```

## 유효성 검증 규칙

1. **redis.enabled = false**일 때 `redis.external.url`은 필수
2. **redis.external.url**은 `redis://` 또는 `rediss://`로 시작해야 함
3. **ingress.tls.enabled = true**일 때 `ingress.tls.secretName`은 필수
4. **namespace**는 DNS-1123 레이블 규칙 준수 (소문자, 숫자, 하이픈만)
5. **cpu**: Kubernetes 리소스 표기법 (예: 100m, 1, 2000m)
6. **memory**: Kubernetes 리소스 표기법 (예: 128Mi, 1Gi, 512Mi)
7. **replicas**: 양수 정수
8. **service.type**: ClusterIP, NodePort, LoadBalancer 중 하나
9. **ingress.pathType**: Prefix 또는 Exact

## 예제 설정

### 최소 설정 (기본값 사용)

**Helm values.yaml**:
```yaml
namespace: default
image:
  repository: ghcr.io/egoavara/realtime-svg
  tag: v0.1.4
ingress:
  host: realtime-svg.mycompany.com
```

### 프로덕션 설정 (외부 Redis)

**Helm values.yaml**:
```yaml
namespace: production
image:
  repository: ghcr.io/egoavara/realtime-svg
  tag: v0.1.4

replicas: 5

resources:
  requests:
    cpu: 500m
    memory: 512Mi
  limits:
    cpu: 2000m
    memory: 2Gi

redis:
  enabled: false
  external:
    url: redis://:mypassword@redis.production.svc.cluster.local:6379/

ingress:
  enabled: true
  host: realtime-svg.mycompany.com
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  tls:
    enabled: true
    secretName: realtime-svg-tls
```

### 개발 설정 (클러스터 내 Redis, NodePort)

**Helm values.yaml**:
```yaml
namespace: dev
image:
  tag: v0.1.4

redis:
  enabled: true

service:
  type: NodePort

ingress:
  enabled: false
```

## 환경별 권장 설정

### Development

- `replicas: 1`
- `redis.enabled: true` (클러스터 내 Redis)
- `service.type: NodePort` (로컬 접근)
- `ingress.enabled: false`
- 리소스 제한 최소

### Staging

- `replicas: 2`
- `redis.enabled: false` (외부 Redis 사용)
- `service.type: ClusterIP`
- `ingress.enabled: true`
- 프로덕션과 유사한 리소스 설정

### Production

- `replicas: 5+` (트래픽에 따라 조정)
- `redis.enabled: false` (관리형 Redis 필수)
- `service.type: ClusterIP`
- `ingress.enabled: true`
- `ingress.tls.enabled: true`
- 충분한 리소스 제한
- 모니터링 및 로깅 활성화
