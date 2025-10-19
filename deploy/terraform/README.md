# realtime-svg Terraform Module

Terraform을 사용하여 realtime-svg 애플리케이션을 Kubernetes에 배포하는 IaC(Infrastructure as Code) 모듈입니다.

## 특징

- ✅ 코드로 관리되는 인프라 (Infrastructure as Code)
- ✅ 상태 관리 및 변경 추적
- ✅ 선언적 설정 방식
- ✅ 일관된 매개변수 구조 (Helm, kubectl, Pulumi와 동일)
- ✅ 조건부 리소스 생성 (Redis, Ingress)
- ✅ 원격 백엔드 지원

## 사전 요구사항

- **Terraform**: 1.0 이상
- **Kubernetes**: 1.24 이상
- **kubectl**: 클러스터 접근 설정 완료

## 빠른 시작

### 1. 초기화

```bash
cd deploy/terraform
terraform init
```

### 2. 설정

```bash
cp terraform.tfvars.example terraform.tfvars
# terraform.tfvars 파일 편집
```

### 3. 배포

```bash
terraform plan    # 변경사항 미리보기
terraform apply   # 배포 실행
```

### 4. 확인

```bash
terraform output
kubectl get all -l app.kubernetes.io/name=realtime-svg
```

## 주요 파일

| 파일 | 설명 |
|------|------|
| `main.tf` | Provider 설정 및 공통 로직 |
| `versions.tf` | Terraform 및 Provider 버전 정의 |
| `variables.tf` | 모든 입력 변수 정의 |
| `outputs.tf` | 출력 값 정의 |
| `deployment.tf` | Deployment 및 Service 리소스 |
| `configmap.tf` | ConfigMap 리소스 |
| `secret.tf` | Secret 리소스 |
| `ingress.tf` | Ingress 리소스 (선택적) |
| `redis.tf` | Redis Deployment 및 Service (선택적) |
| `terraform.tfvars.example` | 설정 예제 파일 |

## 배포 예제

### 개발 환경

```hcl
# terraform.tfvars
namespace = "dev"
replicas  = 1

redis_enabled = true
ingress_enabled = false
service_type = "NodePort"
```

### 프로덕션 환경

```hcl
# terraform.tfvars
namespace = "production"
replicas  = 5

redis_enabled = false
redis_external_url = "redis://:password@redis.prod.svc:6379/"

ingress_enabled = true
ingress_host = "realtime-svg.mycompany.com"
ingress_tls_enabled = true
ingress_tls_secret_name = "realtime-svg-tls"

resources_limits_cpu = "2000m"
resources_limits_memory = "2Gi"
```

## 주요 변수

| 변수 | 기본값 | 설명 |
|------|--------|------|
| `namespace` | `default` | Kubernetes 네임스페이스 |
| `image_repository` | `ghcr.io/egoavara/realtime-svg` | 컨테이너 이미지 |
| `image_tag` | `v0.1.4` | 이미지 태그 |
| `replicas` | `2` | Pod 복제본 수 |
| `redis_enabled` | `true` | 클러스터 내 Redis 배포 |
| `redis_external_url` | `""` | 외부 Redis URL |
| `ingress_enabled` | `true` | Ingress 생성 |
| `service_type` | `ClusterIP` | Service 타입 |
| `service_port` | `80` | Service 포트 |

전체 변수 목록은 `variables.tf` 파일을 참조하세요.

## 출력 값

| 출력 | 설명 |
|------|------|
| `namespace` | 배포된 네임스페이스 |
| `deployment_name` | Deployment 이름 |
| `service_name` | Service 이름 |
| `ingress_url` | Ingress URL (활성화된 경우) |
| `redis_endpoint` | Redis 연결 엔드포인트 |

## 일반적인 작업

### 버전 업데이트

```bash
# terraform.tfvars
image_tag = "v0.1.5"

terraform apply
```

### 복제본 수 변경

```bash
# terraform.tfvars
replicas = 5

terraform apply
```

### 외부 Redis로 전환

```bash
# terraform.tfvars
redis_enabled = false
redis_external_url = "redis://redis.external.svc:6379/"

terraform apply
```

### 리소스 제거

```bash
terraform destroy
```

## 상태 관리

### 로컬 상태 (개발)

기본적으로 로컬 파일 시스템에 상태를 저장합니다.

### 원격 상태 (프로덕션 권장)

S3 백엔드 설정:

```hcl
# backend.tf 생성
terraform {
  backend "s3" {
    bucket = "my-terraform-state"
    key    = "realtime-svg/terraform.tfstate"
    region = "us-west-2"
  }
}
```

초기화:

```bash
terraform init -migrate-state
```

## 모듈로 사용

다른 Terraform 프로젝트에서 이 모듈을 재사용:

```hcl
module "realtime_svg" {
  source = "git::https://github.com/egoavara/realtime-svg.git//deploy/terraform"

  namespace = "production"
  replicas  = 3
  
  redis_enabled = false
  redis_external_url = var.redis_url
  
  ingress_enabled = true
  ingress_host = "realtime-svg.example.com"
}

output "service_endpoint" {
  value = module.realtime_svg.service_name
}
```

## 트러블슈팅

### Provider 인증 오류

```bash
# kubeconfig 확인
kubectl cluster-info

# 환경 변수 설정
export KUBE_CONFIG_PATH=~/.kube/config
```

### 상태 잠금 오류

```bash
# 잠금 해제 (주의: 다른 작업이 실행 중이 아닌지 확인)
terraform force-unlock <LOCK_ID>
```

### 변수 검증 오류

변수 정의에 validation 블록이 포함되어 있습니다:

- `replicas`: 0보다 커야 함
- `service_port`: 1-65535 범위
- `service_type`: ClusterIP, NodePort, LoadBalancer 중 하나
- `ingress_path_type`: Prefix 또는 Exact

## 검증

```bash
# 문법 검사
terraform fmt -check

# 설정 검증
terraform validate

# 계획 확인
terraform plan
```

## 보안 고려사항

1. **상태 파일 보호**
   - `.tfstate` 파일을 Git에 커밋하지 마세요
   - 원격 백엔드를 사용하세요
   - 상태 파일 암호화를 활성화하세요

2. **민감한 정보**
   - `terraform.tfvars`를 `.gitignore`에 추가하세요
   - 비밀번호는 환경 변수로 전달하세요:
     ```bash
     export TF_VAR_redis_password="secret"
     terraform apply
     ```

3. **네임스페이스 분리**
   - 환경별로 별도 네임스페이스 사용
   - Terraform workspace 또는 별도 상태 파일 사용

## 환경별 관리

### Workspace 사용

```bash
# Production workspace
terraform workspace new production
terraform workspace select production
terraform apply -var-file="production.tfvars"

# Staging workspace
terraform workspace new staging
terraform workspace select staging
terraform apply -var-file="staging.tfvars"
```

### 디렉토리 분리

```
terraform/
├── environments/
│   ├── production/
│   │   ├── main.tf
│   │   └── terraform.tfvars
│   ├── staging/
│   │   ├── main.tf
│   │   └── terraform.tfvars
│   └── dev/
│       ├── main.tf
│       └── terraform.tfvars
└── modules/
    └── realtime-svg/
        ├── main.tf
        ├── variables.tf
        └── outputs.tf
```

## 문서

- [Terraform 배포 가이드](../../docs/deployment/terraform-guide.md)
- [설정 레퍼런스](../../docs/deployment/configuration-reference.md)
- [아키텍처 문서](../../docs/deployment/architecture.md)

## 지원

- GitHub Issues: https://github.com/egoavara/realtime-svg/issues
- Documentation: https://github.com/egoavara/realtime-svg/tree/main/docs

## 라이선스

이 프로젝트의 라이선스는 저장소 루트의 LICENSE 파일을 참조하세요.
