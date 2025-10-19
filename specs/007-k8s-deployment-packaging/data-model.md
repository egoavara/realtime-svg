# Data Model: Kubernetes 배포 패키징

**Feature**: 007-k8s-deployment-packaging  
**Date**: 2025-10-19

## Overview

이 기능은 런타임 데이터 모델이 아닌 **설정 데이터 모델**을 정의합니다. 배포 시 사용되는 매개변수 구조와 Kubernetes 리소스 매핑을 문서화합니다.

## Configuration Schema

### DeploymentConfig

모든 배포 도구가 공통으로 사용하는 설정 스키마입니다.

```yaml
DeploymentConfig:
  namespace: string                    # Kubernetes 네임스페이스 (기본: default)
  
  image:                               # 컨테이너 이미지 설정
    repository: string                 # 이미지 저장소 (기본: ghcr.io/egoavara/realtime-svg)
    tag: string                        # 이미지 태그 (기본: v0.1.4)
    pullPolicy: string                 # 풀 정책 (기본: IfNotPresent)
  
  replicas: integer                    # 복제본 수 (기본: 2)
  
  resources:                           # 리소스 제한
    requests:
      cpu: string                      # 기본: 100m
      memory: string                   # 기본: 128Mi
    limits:
      cpu: string                      # 기본: 500m
      memory: string                   # 기본: 512Mi
  
  redis:                               # Redis 설정
    enabled: boolean                   # 클러스터 내 Redis 설치 여부 (기본: true)
    host: string                       # 외부 Redis 호스트 (enabled=false일 때 필수)
    port: integer                      # Redis 포트 (기본: 6379)
    password: string                   # Redis 비밀번호 (선택)
    resources:                         # Redis 리소스 (enabled=true일 때만)
      requests:
        cpu: string                    # 기본: 100m
        memory: string                 # 기본: 128Mi
      limits:
        cpu: string                    # 기본: 200m
        memory: string                 # 기본: 256Mi
  
  ingress:                             # 인그레스 설정
    enabled: boolean                   # 인그레스 생성 여부 (기본: true)
    host: string                       # 호스트명 (기본: realtime-svg.example.com)
    path: string                       # 경로 (기본: /)
    pathType: string                   # 경로 타입 (Prefix|Exact, 기본: Prefix)
    annotations: map[string]string     # 커스텀 annotations (선택)
    tls:
      enabled: boolean                 # TLS 활성화 (기본: false)
      secretName: string               # TLS 시크릿 이름 (enabled=true일 때 필수)
  
  service:                             # 서비스 설정
    type: string                       # 서비스 타입 (ClusterIP|NodePort|LoadBalancer, 기본: ClusterIP)
    port: integer                      # 서비스 포트 (기본: 8080)
  
  config:                              # ConfigMap 데이터
    logLevel: string                   # 로그 레벨 (기본: info)
    port: integer                      # 애플리케이션 포트 (기본: 8080)
  
  secrets:                             # Secret 데이터 (base64 인코딩 필요)
    apiKey: string                     # API 키 (선택)
```

### 유효성 검사 규칙

1. **redis.enabled = false**일 때 `redis.host`는 필수
2. **ingress.tls.enabled = true**일 때 `ingress.tls.secretName`은 필수
3. **namespace**는 DNS-1123 레이블 규칙 준수 (소문자, 숫자, 하이픈만)
4. **cpu**: Kubernetes 리소스 표기법 (예: 100m, 1, 2000m)
5. **memory**: Kubernetes 리소스 표기법 (예: 128Mi, 1Gi, 512Mi)
6. **replicas**: 양수 정수
7. **service.type**: ClusterIP, NodePort, LoadBalancer 중 하나
8. **ingress.pathType**: Prefix 또는 Exact

## Kubernetes 리소스 매핑

### DeploymentConfig → Kubernetes Resources

```
DeploymentConfig
├─ namespace → Namespace (선택적 생성, Helm/Terraform/Pulumi만)
├─ image.repository → Deployment/realtime-svg/spec/template/spec/containers[0]/image (repository 부분)
├─ image.tag → Deployment/realtime-svg/spec/template/spec/containers[0]/image (tag 부분)
├─ image.pullPolicy → Deployment/realtime-svg/spec/template/spec/containers[0]/imagePullPolicy
├─ replicas → Deployment/realtime-svg/spec/replicas
├─ resources → Deployment/realtime-svg/spec/template/spec/containers[0]/resources
├─ redis.enabled=true → Deployment/redis + Service/redis
├─ redis.host → ConfigMap/data/REDIS_HOST
├─ redis.port → ConfigMap/data/REDIS_PORT
├─ redis.password → Secret/data/REDIS_PASSWORD
├─ ingress → Ingress
├─ service → Service/realtime-svg
├─ config → ConfigMap
└─ secrets → Secret
```

### 생성되는 Kubernetes 리소스 목록

1. **Deployment/realtime-svg**: realtime-svg 애플리케이션 (단일 통합 이미지)
2. **Deployment/redis** (조건부): `redis.enabled=true`일 때만
3. **Service/realtime-svg**: realtime-svg 서비스
4. **Service/redis** (조건부): `redis.enabled=true`일 때만
5. **Ingress** (조건부): `ingress.enabled=true`일 때만
6. **ConfigMap**: 환경 변수 설정
7. **Secret**: 민감 정보 저장

## 도구별 매핑 테이블

| 설정 경로 | Helm | Terraform | Pulumi | kubectl/kustomize |
|-----------|------|-----------|--------|-------------------|
| namespace | `.Values.namespace` | `var.namespace` | `config.get("namespace")` | `kustomization.yaml/namespace` |
| image.repository | `.Values.image.repository` | `var.image_repository` | `config.get("image:repository")` | `kustomization.yaml/images[].newName` |
| image.tag | `.Values.image.tag` | `var.image_tag` | `config.get("image:tag")` | `kustomization.yaml/images[].newTag` |
| replicas | `.Values.replicas` | `var.replicas` | `config.getNumber("replicas")` | `kustomization.yaml/replicas[].count` |
| redis.enabled | `.Values.redis.enabled` | `var.redis_enabled` | `config.getBoolean("redis:enabled")` | 별도 파일 포함/제외 |
| ingress.enabled | `.Values.ingress.enabled` | `var.ingress_enabled` | `config.getBoolean("ingress:enabled")` | 별도 파일 포함/제외 |

## 상태 전이

배포 설정은 정적이므로 상태 전이가 없습니다. 단, 배포 프로세스는 다음 단계를 거칩니다:

```
1. 설정 검증
   ↓
2. 템플릿 렌더링 (Helm) / 리소스 생성 (Terraform/Pulumi) / 매니페스트 적용 (kubectl)
   ↓
3. Kubernetes API 제출
   ↓
4. 리소스 생성/업데이트
   ↓
5. Pod 스케줄링 및 시작
   ↓
6. Readiness Probe 통과
   ↓
7. 트래픽 수신 준비 완료
```

## 예제 설정

### 최소 설정 (기본값 사용)

```yaml
# Helm values.yaml
namespace: default
image:
  repository: ghcr.io/egoavara/realtime-svg
  tag: v0.1.4
ingress:
  host: realtime-svg.mycompany.com
```

### 프로덕션 설정 (외부 Redis)

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
  host: redis.production.svc.cluster.local
  port: 6379
  password: <base64-encoded>

ingress:
  enabled: true
  host: realtime-svg.mycompany.com
  path: /
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  tls:
    enabled: true
    secretName: realtime-svg-tls
```

### 개발 설정 (클러스터 내 Redis, NodePort)

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
