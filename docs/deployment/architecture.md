# Deployment Architecture

realtime-svg 애플리케이션의 Kubernetes 배포 아키텍처를 설명합니다.

## 개요

realtime-svg는 Kubernetes에 배포되는 stateless 애플리케이션입니다. 모든 상태는 Redis에 저장되며, 4가지 배포 방법을 지원합니다:

1. **Helm** - 표준 패키지 관리자
2. **kubectl** - 순수 YAML 매니페스트
3. **Terraform** - Infrastructure as Code
4. **Pulumi** - 프로그래밍 방식 IaC

## 아키텍처 다이어그램

```
┌─────────────────────────────────────────────────────────────┐
│                        Internet                             │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │   Ingress Controller  │
              │  (nginx/traefik/etc)  │
              └──────────┬────────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │   Ingress Resource   │
              │  host: realtime-svg  │
              │  path: /             │
              └──────────┬────────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Service (ClusterIP) │
              │   port: 8080         │
              └──────────┬────────────┘
                         │
          ┌──────────────┴──────────────┐
          │                             │
          ▼                             ▼
┌─────────────────┐           ┌─────────────────┐
│  Pod (replica 1) │           │  Pod (replica 2) │
│                 │           │                 │
│  realtime-svg   │           │  realtime-svg   │
│  Container      │           │  Container      │
│                 │           │                 │
│  Port: 8080     │           │  Port: 8080     │
└────────┬────────┘           └────────┬────────┘
         │                             │
         └──────────────┬──────────────┘
                        │
                        ▼
              ┌──────────────────────┐
              │   Redis Service      │
              │   port: 6379         │
              └──────────┬────────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │    Redis Pod         │
              │  (optional,          │
              │   in-memory only)    │
              └──────────────────────┘
```

## 컴포넌트 상세

### 1. Application Pods

- **이미지**: `ghcr.io/egoavara/realtime-svg:v0.1.4`
- **포트**: 8080 (HTTP)
- **복제본**: 2 (기본값, 조정 가능)
- **상태**: Stateless
- **헬스 체크**:
  - Liveness Probe: `/health` (10초마다 체크)
  - Readiness Probe: `/ready` (5초마다 체크)

### 2. Service

- **타입**: ClusterIP (기본값)
- **포트**: 8080
- **선택자**: `app=realtime-svg`
- **기능**: 여러 Pod 간 로드 밸런싱

### 3. Ingress (선택적)

- **활성화**: `ingress.enabled=true`
- **호스트**: 사용자 정의 (예: `realtime-svg.example.com`)
- **경로**: `/` (기본값)
- **TLS**: 선택적 (`ingress.tls.enabled=true`)
- **Annotations**: 인그레스 컨트롤러별 커스터마이징 가능

### 4. ConfigMap

환경 설정을 저장:
- `REDIS_HOST`: Redis 서버 주소
- `REDIS_PORT`: Redis 포트
- `LOG_LEVEL`: 로그 레벨
- `PORT`: 애플리케이션 포트

### 5. Secret

민감 정보를 저장 (base64 인코딩):
- `redis-password`: Redis 비밀번호
- `apiKey`: API 키 (선택적)

### 6. Redis (선택적)

**클러스터 내 Redis** (`redis.enabled=true`):
- **이미지**: `redis:7-alpine`
- **포트**: 6379
- **영속성**: **없음** (인메모리만, Pod 재시작 시 데이터 유실)
- **용도**: 개발/테스트 환경
- **명령**: `redis-server --save "" --appendonly no`

**외부 Redis** (`redis.enabled=false`):
- 프로덕션 환경 권장
- 관리형 Redis 서비스 사용 (AWS ElastiCache, GCP Memorystore, Azure Cache 등)
- 데이터 영속성 및 고가용성 확보

## 네트워크 플로우

### 클라이언트 요청 플로우

1. **클라이언트** → DNS 조회 (`realtime-svg.example.com`)
2. **DNS** → Ingress Controller의 LoadBalancer IP 반환
3. **Ingress Controller** → Host header 확인, Ingress 규칙 매칭
4. **Ingress Resource** → Service로 라우팅
5. **Service** → 사용 가능한 Pod 중 하나로 로드 밸런싱
6. **Pod** → 요청 처리, Redis에서 세션 데이터 조회
7. **Redis** → 세션 데이터 반환
8. **Pod** → 응답 생성 및 반환

### 세션 데이터 플로우

1. **클라이언트** → POST /api/session (새 세션 생성)
2. **Pod** → Redis에 세션 데이터 저장 (SET key value EX ttl)
3. **Redis** → 세션 ID 반환
4. **Pod** → 클라이언트에게 세션 ID 응답

5. **클라이언트** → GET /stream/{session_id} (스트림 구독)
6. **Pod** → Redis에서 세션 데이터 조회 (GET key)
7. **Pod** → Redis pubsub 구독 (SUBSCRIBE channel)
8. **Pod** → 클라이언트에게 multipart/x-mixed-replace 스트리밍 시작

9. **다른 클라이언트** → PUT /api/session/{session_id} (세션 업데이트)
10. **Pod** → Redis에 새 데이터 저장 및 pubsub 발행 (PUBLISH channel message)
11. **Redis** → 모든 구독자에게 메시지 전달
12. **스트리밍 중인 Pod** → 새 프레임 렌더링 및 클라이언트에게 전송

## 리소스 관리

### CPU & Memory

**애플리케이션 Pod** (기본값):
```yaml
resources:
  requests:
    cpu: 100m      # 0.1 CPU
    memory: 128Mi  # 128 MiB
  limits:
    cpu: 500m      # 0.5 CPU
    memory: 512Mi  # 512 MiB
```

**Redis Pod** (기본값):
```yaml
resources:
  requests:
    cpu: 100m
    memory: 128Mi
  limits:
    cpu: 200m
    memory: 256Mi
```

### 오토스케일링 (Phase 7에서 추가 예정)

현재 버전은 수동 스케일링만 지원:
```bash
# Helm
helm upgrade realtime-svg ./deploy/helm/realtime-svg --set replicas=5

# kubectl
kubectl scale deployment realtime-svg --replicas=5

# Terraform
# terraform.tfvars에서 replicas=5로 수정 후
terraform apply

# Pulumi
pulumi config set replicas 5
pulumi up
```

## 보안 고려사항

### 네트워크 정책 (Scope 외)

현재 버전은 기본 Kubernetes 네트워크 정책을 사용합니다. 프로덕션에서는 다음을 고려하세요:
- Pod 간 통신 제한
- Ingress/Egress 규칙 정의
- NetworkPolicy 리소스 적용

### Secret 관리

- **개발**: Kubernetes Secret (base64 인코딩)
- **프로덕션 권장**:
  - Sealed Secrets (bitnami-labs/sealed-secrets)
  - External Secrets Operator
  - HashiCorp Vault
  - Cloud provider secret managers (AWS Secrets Manager, GCP Secret Manager 등)

### TLS/HTTPS

- **인그레스에서 TLS 종료**
- cert-manager를 사용한 자동 인증서 관리 권장
- Let's Encrypt 무료 인증서 활용 가능

## 배포 일관성

모든 배포 방법 (Helm, kubectl, Terraform, Pulumi)은 **동일한 Kubernetes 리소스**를 생성합니다:

| 리소스 | 수량 | 조건 |
|--------|------|------|
| Deployment (realtime-svg) | 1 | 항상 |
| Service (realtime-svg) | 1 | 항상 |
| ConfigMap | 1 | 항상 |
| Secret | 1 | 항상 |
| Ingress | 0-1 | `ingress.enabled=true`일 때 |
| Deployment (redis) | 0-1 | `redis.enabled=true`일 때 |
| Service (redis) | 0-1 | `redis.enabled=true`일 때 |

## 모니터링 및 로깅 (Scope 외)

현재 버전은 표준 출력(stdout)으로 로그를 출력합니다. 프로덕션에서는 다음을 고려하세요:

- **로그 수집**: Fluent Bit, Fluentd, Logstash
- **메트릭**: Prometheus, Grafana
- **추적**: Jaeger, Zipkin
- **대시보드**: Kubernetes Dashboard, Lens

## 고가용성

### 애플리케이션 레벨

- **복제본**: 최소 2개 이상 (프로덕션에서는 3-5개 권장)
- **Pod Disruption Budget**: 최소 1개 Pod 항상 사용 가능
- **Rolling Update**: 무중단 배포 (maxUnavailable=1, maxSurge=1)
- **Readiness Probe**: 트래픽 수신 전 준비 상태 확인

### Redis 레벨

- **클러스터 내 Redis**: 고가용성 없음 (단일 Pod)
- **외부 Redis**: 관리형 서비스의 고가용성 기능 활용
  - Redis Sentinel (자동 페일오버)
  - Redis Cluster (샤딩 및 복제)
  - 클라우드 관리형 서비스의 자동 백업 및 복구

## 재해 복구

### 백업

- **애플리케이션**: Stateless이므로 백업 불필요, 이미지 저장소만 관리
- **Redis 데이터**: 
  - 클러스터 내 Redis: 백업 없음 (인메모리만)
  - 외부 Redis: 관리형 서비스의 자동 백업 사용

### 복구

- **Pod 장애**: Kubernetes가 자동으로 재시작
- **Node 장애**: Kubernetes가 다른 노드로 Pod 재스케줄링
- **클러스터 장애**: 
  - 멀티 AZ/리전 배포 권장 (Scope 외)
  - 인프라 코드 (Terraform/Pulumi)로 신속한 재배포 가능

## 성능 최적화

### 수평 확장

```yaml
# 트래픽 증가 시
replicas: 10

# 트래픽 감소 시
replicas: 2
```

### 리소스 튜닝

모니터링 데이터를 기반으로 `resources.requests`와 `resources.limits` 조정:

```yaml
# 고부하 환경
resources:
  requests:
    cpu: 500m
    memory: 512Mi
  limits:
    cpu: 2000m
    memory: 2Gi
```

### Redis 연결 풀링

애플리케이션 코드에서 Redis 연결 풀 사용 (기본 설정됨).

## 배포 전략 비교

| 배포 방법 | 난이도 | 유연성 | 사용 사례 |
|-----------|--------|--------|----------|
| **Helm** | 쉬움 | 중간 | 대부분의 팀, 빠른 배포 |
| **kubectl** | 쉬움 | 낮음 | GitOps, 간단한 배포 |
| **Terraform** | 중간 | 높음 | 인프라 코드 통합, 멀티 클라우드 |
| **Pulumi** | 중간 | 매우 높음 | 개발자 친화적, 타입 안전성 |

## 참고 자료

- [Helm Best Practices](https://helm.sh/docs/chart_best_practices/)
- [Kubernetes Deployment Strategies](https://kubernetes.io/docs/concepts/workloads/controllers/deployment/)
- [Redis Kubernetes Operator](https://github.com/spotahome/redis-operator)
- [Ingress Controllers Comparison](https://docs.google.com/spreadsheets/d/191WWNpjJ2za6-nbG4ZoUMXMpUK8KlCIosvQB0f-oq3k)
