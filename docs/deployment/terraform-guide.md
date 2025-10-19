# Terraform Deployment Guide

이 가이드는 Terraform을 사용하여 realtime-svg를 Kubernetes에 배포하는 방법을 설명합니다.

## 사전 요구사항

- Kubernetes 클러스터 (1.24+)
- Terraform 1.0+ 설치
- kubectl 설치 및 클러스터 접근 설정
- kubeconfig 파일 설정 완료

## Terraform 설치

Terraform이 설치되어 있지 않은 경우:

```bash
# macOS
brew install terraform

# Linux (Ubuntu/Debian)
wget -O- https://apt.releases.hashicorp.com/gpg | sudo gpg --dearmor -o /usr/share/keyrings/hashicorp-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/hashicorp-archive-keyring.gpg] https://apt.releases.hashicorp.com $(lsb_release -cs) main" | sudo tee /etc/apt/sources.list.d/hashicorp.list
sudo apt update && sudo apt install terraform

# Windows
choco install terraform

# 설치 확인
terraform version
```

## 빠른 시작

### 1. 저장소 클론

```bash
git clone https://github.com/egoavara/realtime-svg.git
cd realtime-svg/deploy/terraform
```

### 2. Terraform 초기화

```bash
terraform init
```

이 명령은 Kubernetes provider를 다운로드하고 Terraform 작업 디렉토리를 초기화합니다.

### 3. 설정 파일 생성

```bash
cp terraform.tfvars.example terraform.tfvars
# terraform.tfvars 파일을 편집하여 환경에 맞게 수정
```

### 4. 계획 확인

```bash
terraform plan
```

생성될 리소스를 확인합니다.

### 5. 배포 실행

```bash
terraform apply
```

`yes`를 입력하여 배포를 확인합니다.

### 6. 배포 확인

```bash
# 출력 값 확인
terraform output

# Kubernetes 리소스 확인
kubectl get all -n default -l app.kubernetes.io/name=realtime-svg
```

## 배포 시나리오

### 개발 환경 (in-cluster Redis)

```hcl
# terraform.tfvars
namespace = "dev"
replicas  = 1

redis_enabled = true
redis_password = ""

ingress_enabled = false

service_type = "NodePort"
```

```bash
terraform apply
```

### 스테이징 환경 (외부 Redis)

```hcl
# terraform.tfvars
namespace = "staging"
replicas  = 2

redis_enabled = false
redis_external_url = "redis://redis.staging.svc:6379/"

ingress_enabled = true
ingress_host = "realtime-svg-staging.example.com"

service_type = "ClusterIP"
```

```bash
terraform apply
```

### 프로덕션 환경 (외부 Redis + TLS)

```hcl
# terraform.tfvars
namespace = "production"
replicas  = 5

resources_requests_cpu = "500m"
resources_requests_memory = "512Mi"
resources_limits_cpu = "2000m"
resources_limits_memory = "2Gi"

redis_enabled = false
redis_external_url = "redis://:my-password@redis.production.svc:6379/"

ingress_enabled = true
ingress_host = "realtime-svg.mycompany.com"
ingress_tls_enabled = true
ingress_tls_secret_name = "realtime-svg-tls"

ingress_annotations = {
  "cert-manager.io/cluster-issuer" = "letsencrypt-prod"
  "nginx.ingress.kubernetes.io/ssl-redirect" = "true"
}

service_type = "ClusterIP"
service_port = 80
```

```bash
terraform apply
```

## 변수 설정 방법

### Option 1: terraform.tfvars 파일 사용 (권장)

```hcl
# terraform.tfvars
namespace = "production"
replicas  = 3
image_tag = "v0.1.5"
```

```bash
terraform apply
```

### Option 2: 명령줄 인자 사용

```bash
terraform apply \
  -var="namespace=production" \
  -var="replicas=3" \
  -var="image_tag=v0.1.5"
```

### Option 3: 환경 변수 사용

```bash
export TF_VAR_namespace="production"
export TF_VAR_replicas=3
export TF_VAR_image_tag="v0.1.5"

terraform apply
```

### Option 4: 다른 변수 파일 사용

```bash
terraform apply -var-file="production.tfvars"
```

## 주요 변수

| 변수 | 설명 | 기본값 |
|------|------|--------|
| `namespace` | Kubernetes 네임스페이스 | `default` |
| `image_repository` | 컨테이너 이미지 저장소 | `ghcr.io/egoavara/realtime-svg` |
| `image_tag` | 이미지 태그 | `v0.1.4` |
| `replicas` | Pod 복제본 수 | `2` |
| `redis_enabled` | 클러스터 내 Redis 배포 여부 | `true` |
| `redis_external_url` | 외부 Redis URL (redis_enabled=false일 때) | `""` |
| `ingress_enabled` | Ingress 생성 여부 | `true` |
| `ingress_host` | Ingress 호스트명 | `realtime-svg.example.com` |
| `service_type` | Service 타입 | `ClusterIP` |

전체 변수 목록은 [configuration-reference.md](./configuration-reference.md)를 참조하세요.

## 업데이트

### 애플리케이션 버전 업데이트

```bash
# terraform.tfvars 파일에서 image_tag 수정
# image_tag = "v0.1.5"

terraform apply
```

### 설정 변경

```bash
# terraform.tfvars 파일 수정
nano terraform.tfvars

# 변경사항 미리보기
terraform plan

# 적용
terraform apply
```

### 선택적 리소스 업데이트

```bash
# 특정 리소스만 업데이트
terraform apply -target=kubernetes_deployment.realtime_svg
```

## 상태 관리

### 원격 백엔드 설정 (권장)

프로덕션 환경에서는 원격 백엔드를 사용하여 상태를 관리하세요.

```hcl
# backend.tf
terraform {
  backend "s3" {
    bucket = "my-terraform-state"
    key    = "realtime-svg/terraform.tfstate"
    region = "us-west-2"
  }
}
```

또는 Terraform Cloud:

```hcl
# backend.tf
terraform {
  backend "remote" {
    organization = "my-org"

    workspaces {
      name = "realtime-svg-production"
    }
  }
}
```

### 상태 확인

```bash
# 현재 상태 확인
terraform show

# 특정 리소스 상태 확인
terraform state show kubernetes_deployment.realtime_svg
```

## 애플리케이션 접근

### Port-Forward로 접근

```bash
kubectl port-forward svc/realtime-svg 8080:80 -n default
# http://localhost:8080 방문
```

### Ingress를 통한 접근

```bash
# Ingress 정보 확인
terraform output ingress_url

# 또는
kubectl get ingress realtime-svg -n default
```

## 제거

### 전체 리소스 제거

```bash
terraform destroy
```

`yes`를 입력하여 제거를 확인합니다.

### 선택적 리소스 제거

```bash
# Ingress만 제거
terraform destroy -target=kubernetes_ingress_v1.realtime_svg
```

## 트러블슈팅

### Provider 인증 오류

**문제**: `Error: Kubernetes cluster unreachable`

**해결**:
```bash
# kubeconfig 확인
kubectl cluster-info

# Terraform이 올바른 kubeconfig를 사용하는지 확인
export KUBE_CONFIG_PATH=~/.kube/config
```

### Redis 연결 오류

**문제**: Pod가 Redis에 연결할 수 없음

**해결**:
```bash
# in-cluster Redis 사용 시
# Redis Pod 확인
kubectl get pods -l app.kubernetes.io/component=redis

# 외부 Redis 사용 시
# redis_external_url 변수 확인
terraform console
> var.redis_external_url
```

### 상태 잠금 오류

**문제**: `Error acquiring the state lock`

**해결**:
```bash
# 잠금 해제 (주의: 다른 작업이 실행 중이지 않은지 확인)
terraform force-unlock <LOCK_ID>
```

### 이미지 풀 오류

**문제**: `ErrImagePull` 또는 `ImagePullBackOff`

**해결**:
```bash
# 이미지 저장소와 태그 확인
terraform console
> var.image_repository
> var.image_tag

# 이미지가 존재하는지 확인
docker pull ghcr.io/egoavara/realtime-svg:v0.1.4
```

## 출력 값

배포 후 다음 출력 값을 확인할 수 있습니다:

```bash
terraform output
```

- `namespace`: 리소스가 배포된 네임스페이스
- `deployment_name`: Deployment 이름
- `service_name`: Service 이름
- `service_type`: Service 타입
- `ingress_url`: Ingress URL (활성화된 경우)
- `redis_endpoint`: Redis 연결 엔드포인트

## 모범 사례

### 1. 상태 파일 보안

- `.tfstate` 파일을 Git에 커밋하지 마세요
- 원격 백엔드를 사용하여 상태를 관리하세요
- 상태 파일 암호화를 활성화하세요

### 2. 변수 파일 관리

- `terraform.tfvars`를 `.gitignore`에 추가하세요
- 민감한 정보는 환경 변수로 전달하세요
- `terraform.tfvars.example`을 커밋하여 템플릿을 제공하세요

### 3. 네임스페이스 분리

```hcl
# 환경별 네임스페이스 사용
namespace = "production"  # 또는 "staging", "dev"
```

### 4. 리소스 태그

```hcl
labels = {
  environment = "production"
  team        = "platform"
  managed-by  = "terraform"
}
```

### 5. 계획 검토

```bash
# 항상 apply 전에 plan 확인
terraform plan -out=tfplan
terraform show tfplan
terraform apply tfplan
```

## 고급 사용법

### Workspace 사용

여러 환경을 관리하려면 Terraform workspace를 사용하세요:

```bash
# 새 workspace 생성
terraform workspace new production
terraform workspace new staging

# workspace 전환
terraform workspace select production

# 현재 workspace 확인
terraform workspace show

# workspace별 변수 파일 사용
terraform apply -var-file="$(terraform workspace show).tfvars"
```

### 모듈로 재사용

다른 프로젝트에서 이 Terraform 설정을 모듈로 사용:

```hcl
module "realtime_svg" {
  source = "git::https://github.com/egoavara/realtime-svg.git//deploy/terraform"

  namespace = "production"
  replicas  = 3
  
  redis_enabled = false
  redis_external_url = "redis://redis.production.svc:6379/"
}
```

### 조건부 리소스

```hcl
# Redis를 특정 조건에서만 배포
redis_enabled = var.environment != "production"
```

## 참고 자료

- [Terraform Kubernetes Provider 문서](https://registry.terraform.io/providers/hashicorp/kubernetes/latest/docs)
- [Terraform 모범 사례](https://www.terraform.io/docs/language/guides/best-practices.html)
- [Configuration Reference](./configuration-reference.md)
- [Architecture Overview](./architecture.md)
