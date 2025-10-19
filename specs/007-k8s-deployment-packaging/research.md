# Research: Kubernetes 배포 패키징

**Feature**: 007-k8s-deployment-packaging  
**Date**: 2025-10-19  
**Status**: Complete

## Research Tasks

### 1. 일관된 매개변수 구조 설계

**목표**: Helm, Terraform, Pulumi, kubectl에서 공통으로 사용할 매개변수 이름과 구조 정의

**결과**:

#### 표준 매개변수 구조

모든 배포 도구에서 다음 구조를 사용:

```yaml
# 네임스페이스
namespace: default

# 컨테이너 이미지 (통합 이미지)
image:
  repository: ghcr.io/egoavara/realtime-svg
  tag: v0.1.4
  pullPolicy: IfNotPresent

# 복제본 수
replicas: 2

# 리소스 제한
resources:
  requests:
    cpu: 100m
    memory: 128Mi
  limits:
    cpu: 500m
    memory: 512Mi

# Redis 설정
redis:
  enabled: true  # false면 외부 Redis 사용
  host: ""  # 외부 Redis 호스트 (enabled=false일 때 필수)
  port: 6379
  password: ""
  resources:
    requests:
      cpu: 100m
      memory: 128Mi
    limits:
      cpu: 200m
      memory: 256Mi

# 인그레스 설정
ingress:
  enabled: true
  host: realtime-svg.example.com
  path: /
  pathType: Prefix
  annotations: {}
  tls:
    enabled: false
    secretName: ""

# 서비스 설정
service:
  backend:
    type: ClusterIP
    port: 8080
  frontend:
    type: ClusterIP
    port: 80

# 환경 변수 (ConfigMap)
config:
  backendUrl: http://backend:8080
  logLevel: info

# 시크릿 (Secret)
secrets:
  apiKey: ""  # base64 인코딩된 값
```

#### 도구별 매핑

**Helm values.yaml**: 위 구조 그대로 사용

**Terraform variables.tf**: 
```hcl
variable "namespace" { default = "default" }
variable "image_repository" { default = "ghcr.io/egoavara/realtime-svg" }
variable "image_tag" { default = "v0.1.4" }
variable "replicas" { default = 2 }
variable "redis_enabled" { default = true }
variable "ingress_enabled" { default = true }
variable "ingress_host" { default = "realtime-svg.example.com" }
# ... 등등
```

**Pulumi Config**:
```typescript
const config = new pulumi.Config();
const namespace = config.get("namespace") || "default";
const imageRepository = config.get("image:repository") || "ghcr.io/egoavara/realtime-svg";
const imageTag = config.get("image:tag") || "v0.1.4";
const redisEnabled = config.getBoolean("redis:enabled") ?? true;
// ... 등등
```

**kubectl manifests**: 환경 변수나 kustomize overlays 사용
```yaml
# kustomization.yaml
namespace: default
images:
  - name: realtime-svg
    newName: ghcr.io/egoavara/realtime-svg
    newTag: v0.1.4
replicas:
  - name: realtime-svg
    count: 2
```

**Rationale**: 
- Helm의 계층적 values 구조를 기준으로 설계 (가장 직관적)
- Terraform은 flat variable 구조 선호하므로 언더스코어로 계층 표현
- Pulumi는 콜론(`:`)으로 중첩 표현 가능
- kubectl은 kustomize로 패치 적용
- **단일 통합 이미지** 사용으로 frontend/backend 분리 불필요

**Alternatives Considered**:
- Flat 구조 (redis_enabled, app_replicas): 일관성 있지만 그룹화 어려움
- 완전 중첩 구조: Terraform 변환 복잡
- Frontend/Backend 분리 이미지: 현재 통합 이미지(ghcr.io/egoavara/realtime-svg)를 사용하므로 불필요

---

### 2. Helm 차트 모범 사례

**목표**: Helm 차트 작성 시 따라야 할 표준과 패턴 조사

**결과**:

#### 필수 파일
- `Chart.yaml`: 메타데이터 (name, version, appVersion, description)
- `values.yaml`: 기본 설정값
- `templates/`: Kubernetes 리소스 템플릿
- `templates/_helpers.tpl`: 공통 템플릿 헬퍼 함수
- `README.md`: 사용법 및 설정 옵션

#### 모범 사례
1. **레이블 표준화**: `app.kubernetes.io/name`, `app.kubernetes.io/instance`, `app.kubernetes.io/version`
2. **주석 활용**: `helm.sh/hook`, `helm.sh/resource-policy`
3. **조건부 리소스**: `{{ if .Values.redis.enabled }}`로 선택적 배포
4. **네이밍**: `{{ include "realtime-svg.fullname" . }}-backend` 형식
5. **유효성 검사**: `required` 함수로 필수 값 강제
6. **Linting**: `helm lint` 통과 필수

**참고 자료**:
- Helm Best Practices: https://helm.sh/docs/chart_best_practices/
- Artifact Hub 예제: bitnami/redis, nginx-ingress

---

### 3. Kubernetes 리소스 모범 사례

**목표**: Deployment, Service, Ingress 등 리소스 작성 표준

**결과**:

#### Deployment
```yaml
spec:
  replicas: 2
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 1
  selector:
    matchLabels:
      app: backend
  template:
    spec:
      containers:
      - name: backend
        image: realtime-svg/backend:latest
        ports:
        - containerPort: 8080
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
          limits:
            cpu: 500m
            memory: 512Mi
```

#### Service
```yaml
spec:
  type: ClusterIP
  selector:
    app: backend
  ports:
  - port: 8080
    targetPort: 8080
    protocol: TCP
```

#### Ingress
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  annotations:
    # 범용 annotations (대부분 컨트롤러 지원)
    nginx.ingress.kubernetes.io/rewrite-target: /
    cert-manager.io/cluster-issuer: letsencrypt
spec:
  ingressClassName: nginx  # 명시적 지정 권장
  rules:
  - host: realtime-svg.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: frontend
            port:
              number: 80
      - path: /api
        pathType: Prefix
        backend:
          service:
            name: backend
            port:
              number: 8080
```

**중요 사항**:
- `apiVersion: networking.k8s.io/v1` 사용 (v1beta1 deprecated)
- `pathType` 명시 (Prefix/Exact)
- `ingressClassName` 지정 권장

---

### 4. Terraform Kubernetes Provider 패턴

**목표**: Terraform으로 K8s 리소스 관리 시 권장 패턴

**결과**:

#### Provider 설정
```hcl
terraform {
  required_version = ">= 1.0"
  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.23"
    }
  }
}

provider "kubernetes" {
  config_path = "~/.kube/config"
  # 또는
  # host                   = var.cluster_host
  # token                  = var.cluster_token
  # cluster_ca_certificate = base64decode(var.cluster_ca)
}
```

#### 리소스 정의
```hcl
resource "kubernetes_deployment" "backend" {
  metadata {
    name      = "backend"
    namespace = var.namespace
    labels = {
      app = "backend"
    }
  }
  
  spec {
    replicas = var.replicas_backend
    
    selector {
      match_labels = {
        app = "backend"
      }
    }
    
    template {
      metadata {
        labels = {
          app = "backend"
        }
      }
      
      spec {
        container {
          name  = "backend"
          image = "${var.image_backend_repository}:${var.image_backend_tag}"
          
          port {
            container_port = 8080
          }
          
          resources {
            requests = {
              cpu    = var.resources_backend_requests_cpu
              memory = var.resources_backend_requests_memory
            }
            limits = {
              cpu    = var.resources_backend_limits_cpu
              memory = var.resources_backend_limits_memory
            }
          }
        }
      }
    }
  }
}
```

**모범 사례**:
- 변수는 `variables.tf`에 집중
- 출력은 `outputs.tf`에 (service endpoints 등)
- 리소스별 파일 분리 (`backend.tf`, `frontend.tf`)
- `terraform fmt`, `terraform validate` 필수

---

### 5. Pulumi Kubernetes SDK 패턴

**목표**: Pulumi로 K8s 리소스 프로그래밍 방식 관리

**결과**:

#### TypeScript 예제
```typescript
import * as pulumi from "@pulumi/pulumi";
import * as k8s from "@pulumi/kubernetes";

const config = new pulumi.Config();
const namespace = config.get("namespace") || "default";
const backendReplicas = config.getNumber("replicas:backend") || 2;

const backendDeployment = new k8s.apps.v1.Deployment("backend", {
  metadata: {
    name: "backend",
    namespace: namespace,
    labels: {
      app: "backend",
    },
  },
  spec: {
    replicas: backendReplicas,
    selector: {
      matchLabels: {
        app: "backend",
      },
    },
    template: {
      metadata: {
        labels: {
          app: "backend",
        },
      },
      spec: {
        containers: [{
          name: "backend",
          image: config.get("image:backend:repository") + ":" + config.get("image:backend:tag"),
          ports: [{ containerPort: 8080 }],
          resources: {
            requests: {
              cpu: config.get("resources:backend:requests:cpu") || "100m",
              memory: config.get("resources:backend:requests:memory") || "128Mi",
            },
            limits: {
              cpu: config.get("resources:backend:limits:cpu") || "500m",
              memory: config.get("resources:backend:limits:memory") || "512Mi",
            },
          },
        }],
      },
    },
  },
});

export const backendName = backendDeployment.metadata.name;
```

**모범 사례**:
- 타입 안정성 활용 (TypeScript 권장)
- Config 객체로 설정 외부화
- Export로 리소스 정보 노출
- Component Resource로 재사용 가능한 추상화

**Python 대안**:
```python
import pulumi
import pulumi_kubernetes as k8s

config = pulumi.Config()
namespace = config.get("namespace") or "default"

backend = k8s.apps.v1.Deployment(
    "backend",
    metadata=k8s.meta.v1.ObjectMetaArgs(
        name="backend",
        namespace=namespace,
    ),
    spec=k8s.apps.v1.DeploymentSpecArgs(
        replicas=config.get_int("replicas.backend") or 2,
        # ...
    ),
)
```

---

### 6. 선택적 Redis 설치 패턴

**목표**: 사용자가 Redis 포함/제외를 선택 가능하도록 설계

**결과**:

#### 패턴: Conditional Deployment

**Helm**:
```yaml
# values.yaml
redis:
  enabled: true
  host: ""  # 외부 Redis 사용 시 호스트 지정

# templates/deployment-redis.yaml
{{- if .Values.redis.enabled }}
apiVersion: apps/v1
kind: Deployment
metadata:
  name: redis
spec:
  replicas: 1
  template:
    spec:
      containers:
      - name: redis
        image: redis:7-alpine
        command: ["redis-server", "--save", "", "--appendonly", "no"]
        ports:
        - containerPort: 6379
{{- end }}

# templates/configmap.yaml
apiVersion: v1
kind: ConfigMap
data:
  REDIS_HOST: {{ if .Values.redis.enabled }}"redis"{{ else }}{{ required "redis.host required when redis.enabled=false" .Values.redis.host | quote }}{{ end }}
```

**Terraform**:
```hcl
variable "redis_enabled" {
  default = true
}

variable "redis_host" {
  default = ""
}

resource "kubernetes_deployment" "redis" {
  count = var.redis_enabled ? 1 : 0
  # ...
}

locals {
  redis_host = var.redis_enabled ? "redis" : var.redis_host
}

resource "kubernetes_config_map" "config" {
  data = {
    REDIS_HOST = local.redis_host
  }
}
```

**Pulumi**:
```typescript
const redisEnabled = config.getBoolean("redis:enabled") ?? true;

let redisHost: pulumi.Input<string>;

if (redisEnabled) {
  const redisDeployment = new k8s.apps.v1.Deployment("redis", {
    // ...
  });
  redisHost = "redis";
} else {
  redisHost = config.require("redis:host");
}

const configMap = new k8s.core.v1.ConfigMap("config", {
  data: {
    REDIS_HOST: redisHost,
  },
});
```

**인메모리 Redis 주의사항**:
- `--save ""`, `--appendonly no`로 영속성 비활성화
- 개발/테스트 용도 명시
- 프로덕션에서는 외부 관리형 Redis 권장
- 문서에 데이터 유실 경고 포함

---

### 7. 통합 테스트 전략

**목표**: 배포 패키지 검증 방법 정의

**결과**:

#### 로컬 클러스터 테스트
```bash
# minikube 사용
minikube start
eval $(minikube docker-env)
docker build -t realtime-svg/backend:test ./crates/backend
docker build -t realtime-svg/frontend:test ./crates/frontend

# Helm 테스트
helm install test ./deploy/helm/realtime-svg \
  --set image.backend.tag=test \
  --set image.frontend.tag=test

kubectl wait --for=condition=ready pod -l app=backend --timeout=60s
curl $(minikube service frontend --url)

helm uninstall test

# kubectl 테스트
kubectl apply -f deploy/kubernetes/
kubectl wait --for=condition=ready pod -l app=backend --timeout=60s
kubectl delete -f deploy/kubernetes/
```

#### CI/CD 통합
```yaml
# .github/workflows/test-deployments.yml
name: Test Deployments
on: [push]
jobs:
  test-helm:
    runs-on: ubuntu-latest
    steps:
      - uses: azure/setup-helm@v3
      - run: helm lint deploy/helm/realtime-svg
      - run: helm template deploy/helm/realtime-svg | kubectl apply --dry-run=client -f -
      
  test-terraform:
    runs-on: ubuntu-latest
    steps:
      - uses: hashicorp/setup-terraform@v2
      - run: terraform -chdir=deploy/terraform validate
      
  test-pulumi:
    runs-on: ubuntu-latest
    steps:
      - uses: pulumi/setup-pulumi@v2
      - run: cd deploy/pulumi && pulumi preview
```

---

## 결론

**해결된 불확실성**:
1. ✅ 일관된 매개변수 구조 설계 완료
2. ✅ 각 배포 도구별 모범 사례 확인
3. ✅ 선택적 Redis 설치 패턴 정의
4. ✅ 통합 테스트 전략 수립

**다음 단계**: Phase 1 Design (data-model.md, contracts/)
