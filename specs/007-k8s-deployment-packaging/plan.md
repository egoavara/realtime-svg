# Implementation Plan: Kubernetes 배포 패키징

**Branch**: `007-k8s-deployment-packaging` | **Date**: 2025-10-19 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/007-k8s-deployment-packaging/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Helm, kubectl, Terraform, Pulumi 4가지 방식으로 realtime-svg 애플리케이션을 Kubernetes에 배포 가능하도록 패키징합니다. 모든 배포 방식은 일관된 매개변수 구조를 사용하며, stateless 애플리케이션으로 동작하고 선택적 Redis 설치를 지원합니다.

## Technical Context

**Language/Version**: YAML (Kubernetes manifests), HCL (Terraform), TypeScript (Pulumi)
**Primary Dependencies**: 
- Helm 3.x (차트 패키징)
- kubectl 1.24+ (매니페스트 배포)
- Terraform 1.0+ with Kubernetes provider (IaC)
- Pulumi 3.x+ with Kubernetes SDK (프로그래밍 방식 IaC)
- Existing container image: ghcr.io/egoavara/realtime-svg:v0.1.4 (통합 이미지)

**Storage**: Redis (외부 또는 클러스터 내 인메모리), 애플리케이션 자체는 stateless (PVC 불필요)
**Testing**: 
- Helm: `helm lint`, `helm template` 출력 검증
- kubectl: `kubectl apply --dry-run=client`
- Terraform: `terraform validate`, `terraform plan`
- Pulumi: `pulumi preview`, 통합 테스트
- 실제 클러스터 배포 테스트 (minikube/kind)

**Target Platform**: Kubernetes 1.24+ (EKS, GKE, AKS, minikube 등)
**Project Type**: Infrastructure packaging (배포 설정 파일 제공)
**Performance Goals**: 
- 배포 완료: 5분 이내 (Helm 기준)
- 헬스 체크 통과: 30초 이내
- 문서 읽고 첫 배포: 10분 이내

**Constraints**: 
- 모든 배포 방식 동일한 최종 리소스 생성
- 일관된 매개변수 이름 사용 (Helm values ↔ Terraform variables ↔ Pulumi config)
- 범용 인그레스 (특정 컨트롤러 종속 회피)
- Stateless 애플리케이션 (PVC 없음)

**Scale/Scope**: 
- 4가지 배포 방식 지원
- 10+ 설정 가능 매개변수 (replicas, resources, Redis, ingress 등)
- 5+ Kubernetes 리소스 타입 (Deployment, Service, Ingress, ConfigMap, Secret, 선택적 Redis Deployment)
- 단일 통합 컨테이너 이미지 사용 (ghcr.io/egoavara/realtime-svg)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Pre-Design Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Workspace Modularity | ✅ PASS | 배포 패키지는 기존 workspace 구조를 변경하지 않음. 새로운 디렉토리 추가만 수행 |
| II. Contract-First API Design | ✅ PASS | 배포 API는 없음. Kubernetes 리소스 스키마가 계약 역할 |
| III. Template-Based SVG Rendering | ✅ PASS | 배포 설정은 SVG 렌더링에 영향 없음 |
| IV. Testing Discipline | ⚠️ VERIFY | Helm lint, kubectl dry-run, terraform validate로 검증. 실제 클러스터 통합 테스트 필요 |
| V. Observability & Debugging | ✅ PASS | 배포된 애플리케이션은 기존 로깅 유지. 배포 자체는 표준 k8s 도구 사용 |
| VI. Simplicity & Incremental Delivery | ✅ PASS | P1(Helm) → P2(kubectl) → P3(Terraform, Pulumi) 순차 개발 |

### Technology Standards Compliance

| Standard | Status | Notes |
|----------|--------|-------|
| Language & Tooling | ✅ PASS | 기존 Rust 코드 미변경. 배포 설정만 추가 |
| Dependencies | ✅ PASS | 새로운 런타임 dependency 없음. 빌드 도구만 추가 (helm, terraform, pulumi) |
| Session Management | ✅ PASS | Redis 설정을 배포 매개변수로 제공. 기존 로직 미변경 |

### Post-Design Re-Check

*Phase 1 완료 후 재검증*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Workspace Modularity | ✅ PASS | `/deploy` 디렉토리 추가만 수행. 기존 crates 미변경 |
| II. Contract-First API Design | ✅ PASS | Dockerfile Contract, GitHub Actions Contract 정의 완료 |
| III. Template-Based SVG Rendering | ✅ PASS | 배포 설정은 SVG 렌더링 로직 미영향 |
| IV. Testing Discipline | ✅ PASS | Helm lint, kubectl dry-run, terraform validate, Pulumi preview, minikube 통합 테스트 정의 |
| V. Observability & Debugging | ✅ PASS | 배포된 애플리케이션은 기존 로깅 유지 |
| VI. Simplicity & Incremental Delivery | ✅ PASS | P1(Helm) → P2(kubectl) → P3(Terraform, Pulumi) 순차 개발 계획 확립 |

**Overall Status**: ✅ APPROVED - 모든 헌법 원칙 준수 확인

## Project Structure

### Documentation (this feature)

```
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```
# 기존 Rust workspace 구조 (변경 없음)
crates/
├── backend/
├── frontend/
└── common/

# 새로 추가되는 배포 패키지 구조
deploy/
├── helm/
│   └── realtime-svg/
│       ├── Chart.yaml
│       ├── values.yaml
│       ├── templates/
│       │   ├── deployment.yaml
│       │   ├── deployment-redis.yaml (선택적)
│       │   ├── service.yaml
│       │   ├── service-redis.yaml (선택적)
│       │   ├── ingress.yaml
│       │   ├── configmap.yaml
│       │   ├── secret.yaml
│       │   └── _helpers.tpl
│       └── README.md
│
├── kubernetes/
│   ├── deployment.yaml
│   ├── redis-deployment.yaml (선택적)
│   ├── service.yaml
│   ├── redis-service.yaml (선택적)
│   ├── ingress.yaml
│   ├── configmap.yaml
│   ├── secret.yaml.example
│   └── README.md
│
├── terraform/
│   ├── main.tf
│   ├── variables.tf
│   ├── outputs.tf
│   ├── deployment.tf (Deployment, Service)
│   ├── redis.tf (선택적 Deployment, Service)
│   ├── ingress.tf
│   ├── configmap.tf
│   ├── secret.tf
│   ├── versions.tf
│   └── README.md
│
└── pulumi/
    ├── index.ts (또는 __main__.py)
    ├── package.json (또는 requirements.txt)
    ├── Pulumi.yaml
    ├── Pulumi.dev.yaml (예제)
    ├── tsconfig.json (TypeScript 사용 시)
    └── README.md

# 문서
docs/
└── deployment/
    ├── helm-guide.md
    ├── kubectl-guide.md
    ├── terraform-guide.md
    ├── pulumi-guide.md
    └── configuration-reference.md
```

**Structure Decision**: 배포 패키지는 기존 Rust workspace와 독립적으로 `/deploy` 디렉토리에 구성됩니다. 각 배포 방식별로 하위 디렉토리를 생성하여 관련 파일을 그룹화합니다. 문서는 `/docs/deployment`에 배치하여 중앙화합니다.

## Complexity Tracking

*Fill ONLY if Constitution Check has violations that must be justified*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |

