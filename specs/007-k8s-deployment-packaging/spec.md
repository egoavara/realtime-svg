# Feature Specification: Kubernetes 배포 패키징

**Feature Branch**: `007-k8s-deployment-packaging`  
**Created**: 2025-10-19  
**Status**: Draft  
**Input**: User description: "helm, pulumi, terraform kube apply -f 로 쉽게 내 프로젝트를 k8s 에 설치 가능하도록 패키지화 하고 싶어"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Helm으로 원클릭 설치 (Priority: P1)

DevOps 엔지니어가 Helm 차트를 사용하여 realtime-svg 애플리케이션을 Kubernetes 클러스터에 단일 명령어로 설치할 수 있다.

**Why this priority**: Helm은 Kubernetes 패키지 관리의 사실상 표준이며, 가장 널리 사용되는 배포 방법입니다. 많은 조직이 이미 Helm 기반 워크플로우를 운영 중이므로 즉시 채택 가능합니다.

**Independent Test**: `helm install realtime-svg ./charts/realtime-svg` 명령 실행 후 애플리케이션이 정상적으로 배포되고 접근 가능한지 확인하여 독립적으로 테스트 가능합니다.

**Acceptance Scenarios**:

1. **Given** Helm이 설치된 Kubernetes 클러스터, **When** `helm install realtime-svg ./charts/realtime-svg` 실행, **Then** 모든 필요한 리소스(Deployment, Service, ConfigMap 등)가 생성되고 애플리케이션이 정상 작동
2. **Given** 설치된 애플리케이션, **When** `helm upgrade realtime-svg ./charts/realtime-svg --set replicas=3` 실행, **Then** 설정이 변경되고 다운타임 없이 업그레이드 완료
3. **Given** 설치된 애플리케이션, **When** `helm uninstall realtime-svg` 실행, **Then** 모든 관련 리소스가 깔끔하게 제거됨

---

### User Story 2 - kubectl apply로 직접 배포 (Priority: P2)

Kubernetes 관리자가 표준 YAML 매니페스트 파일을 사용하여 `kubectl apply -f` 명령으로 애플리케이션을 배포할 수 있다.

**Why this priority**: Helm을 사용하지 않거나 간단한 배포를 선호하는 사용자를 위한 대안입니다. GitOps 워크플로우에서도 유용합니다.

**Independent Test**: `kubectl apply -f k8s/` 명령 실행 후 모든 리소스가 생성되고 애플리케이션이 정상 작동하는지 확인하여 독립적으로 테스트 가능합니다.

**Acceptance Scenarios**:

1. **Given** Kubernetes 클러스터, **When** `kubectl apply -f k8s/` 실행, **Then** 모든 매니페스트가 적용되고 애플리케이션이 정상 작동
2. **Given** 배포된 애플리케이션, **When** 매니페스트 파일 수정 후 `kubectl apply -f k8s/` 재실행, **Then** 변경사항이 반영됨
3. **Given** 배포된 애플리케이션, **When** `kubectl delete -f k8s/` 실행, **Then** 모든 리소스가 제거됨

---

### User Story 3 - Terraform으로 인프라형 코드 배포 (Priority: P3)

인프라 엔지니어가 Terraform을 사용하여 Kubernetes 리소스를 코드로 관리하고 배포할 수 있다.

**Why this priority**: Terraform을 이미 사용 중인 조직에서 기존 인프라 코드와 통합하여 관리할 수 있습니다. 멀티 클라우드 환경에 유용합니다.

**Independent Test**: `terraform apply` 실행 후 Kubernetes 리소스가 생성되고 애플리케이션이 정상 작동하는지 확인하여 독립적으로 테스트 가능합니다.

**Acceptance Scenarios**:

1. **Given** Terraform이 설치된 환경, **When** `terraform init && terraform apply` 실행, **Then** 모든 Kubernetes 리소스가 생성되고 애플리케이션이 정상 작동
2. **Given** 배포된 리소스, **When** Terraform 설정 변경 후 `terraform apply` 실행, **Then** 변경사항이 반영되고 상태가 일치함
3. **Given** 배포된 리소스, **When** `terraform destroy` 실행, **Then** 모든 리소스가 제거됨

---

### User Story 4 - Pulumi로 프로그래밍 방식 배포 (Priority: P3)

개발자가 Pulumi를 사용하여 익숙한 프로그래밍 언어로 Kubernetes 배포를 작성하고 실행할 수 있다.

**Why this priority**: 개발자 친화적인 배포 방식을 선호하는 팀에 유용합니다. TypeScript/Python 등의 언어로 타입 안정성과 재사용성을 확보할 수 있습니다.

**Independent Test**: `pulumi up` 실행 후 리소스가 생성되고 애플리케이션이 정상 작동하는지 확인하여 독립적으로 테스트 가능합니다.

**Acceptance Scenarios**:

1. **Given** Pulumi가 설치된 환경, **When** `pulumi up` 실행, **Then** 모든 Kubernetes 리소스가 생성되고 애플리케이션이 정상 작동
2. **Given** 배포된 리소스, **When** Pulumi 코드 수정 후 `pulumi up` 실행, **Then** 변경사항이 반영됨
3. **Given** 배포된 리소스, **When** `pulumi destroy` 실행, **Then** 모든 리소스가 제거됨

---

### Edge Cases

- 사용자가 Redis 연결 정보를 제공하지 않았을 때 클러스터 내 Redis를 자동으로 배포할지, 아니면 오류로 처리할지?
- 동일한 네임스페이스에 이미 같은 이름의 리소스가 존재할 때는 어떻게 처리하나?
- 클러스터 내 Redis를 사용하는 경우, Redis Pod가 재시작되면 데이터가 유실되는데 이를 사용자에게 어떻게 안내할지?
- 외부 Redis가 사용 불가능할 때 애플리케이션은 어떻게 동작해야 하나?
- 멀티 아키텍처(amd64, arm64) 지원이 필요한가?
- 다양한 Kubernetes 버전(1.24+, 1.28+) 호환성은 어떻게 보장하나?
- 인그레스 없이 NodePort나 LoadBalancer만으로 접근하는 경우도 지원해야 하나?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: 시스템은 Helm 차트를 제공해야 하며, 표준 `helm install` 명령으로 설치 가능해야 함
- **FR-002**: 시스템은 순수 Kubernetes YAML 매니페스트를 제공해야 하며, `kubectl apply -f` 명령으로 배포 가능해야 함
- **FR-003**: 시스템은 Terraform 모듈을 제공해야 하며, `terraform apply` 명령으로 배포 가능해야 함
- **FR-004**: 시스템은 Pulumi 프로그램을 제공해야 하며, `pulumi up` 명령으로 배포 가능해야 함
- **FR-005**: 모든 배포 방식은 사용자가 설정값(replicas, resource limits, Redis 연결 정보 등)을 커스터마이징 할 수 있어야 함
- **FR-006**: 배포 패키지는 frontend, backend 컴포넌트를 포함하고 각각의 서비스와 인그레스 설정을 지원해야 함
- **FR-007**: 배포 패키지는 Redis 의존성 설정을 포함해야 하며, 외부 Redis 또는 클러스터 내 Redis 중 선택 가능해야 함
- **FR-008**: 배포 패키지는 ConfigMap을 통한 환경 설정 주입을 지원해야 함
- **FR-009**: 배포 패키지는 Secret을 통한 민감 정보(API 키, 비밀번호 등) 관리를 지원해야 함
- **FR-010**: 모든 배포 방식은 동일한 최종 상태(동일한 Kubernetes 리소스)를 생성해야 함
- **FR-011**: 배포 문서는 각 배포 방식의 사전 요구사항, 설치 단계, 설정 옵션을 명확히 설명해야 함
- **FR-012**: 배포 패키지는 헬스 체크 및 리소스 준비 상태 확인(readiness probe, liveness probe)을 포함해야 함
- **FR-013**: 배포 패키지는 리소스 제한(CPU, 메모리)의 기본값을 제공하되 오버라이드 가능해야 함
- **FR-014**: 애플리케이션은 완전히 stateless로 작동해야 하며, 영속성 볼륨(PVC)을 사용하지 않음 (모든 상태는 Redis에 저장)
- **FR-015**: 배포 패키지는 범용 Ingress 리소스를 제공해야 하며, 표준 annotations를 사용하여 대부분의 인그레스 컨트롤러와 호환되어야 함
- **FR-016**: kubectl 배포는 기본 네임스페이스(default)를 사용해야 하며, Helm, Terraform, Pulumi 배포는 기본값으로 default 네임스페이스를 사용하되 사용자 설정으로 다른 네임스페이스를 지정 가능해야 함
- **FR-017**: 배포 패키지는 선택적으로 Redis를 클러스터 내에 인메모리 방식으로 설치할 수 있는 옵션을 제공해야 하며, 사용자가 Redis 설치를 제외하고도 배포 가능해야 함 (외부 Redis 사용 시)
- **FR-018**: 모든 배포 방식(Helm, Terraform, Pulumi, kubectl)은 일관된 매개변수 이름과 구조를 사용해야 하며, 동일한 설정을 다른 배포 도구로 쉽게 변환 가능해야 함

### Key Entities

- **Helm Chart**: realtime-svg 애플리케이션의 Helm 패키지, values.yaml을 통한 설정 커스터마이징 포함
- **Kubernetes Manifests**: Deployment, Service, ConfigMap, Secret, Ingress 등의 YAML 파일 세트
- **Terraform Module**: Kubernetes 프로바이더를 사용한 리소스 정의
- **Pulumi Program**: TypeScript 또는 Python으로 작성된 Kubernetes 리소스 정의
- **Configuration Values**: 복제본 수, 리소스 제한, 환경 변수, Redis 연결 정보 등 사용자 설정 가능 파라미터
- **Container Images**: GitHub Container Registry에 게시된 통합 이미지 (ghcr.io/egoavara/realtime-svg:v0.1.4)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 사용자가 Helm 차트를 사용하여 5분 이내에 애플리케이션을 클러스터에 배포하고 접근 가능한 상태로 만들 수 있음
- **SC-002**: 사용자가 kubectl, Terraform, Pulumi 중 어떤 방식을 사용하든 동일한 기능을 가진 애플리케이션이 배포됨
- **SC-003**: 각 배포 방식의 문서를 읽고 처음 사용하는 사용자가 10분 이내에 성공적으로 배포 완료
- **SC-004**: 배포된 애플리케이션이 헬스 체크를 통과하고 30초 이내에 트래픽을 받을 준비 완료
- **SC-005**: 배포 패키지가 최소 3개 이상의 주요 Kubernetes 배포판(예: EKS, GKE, AKS, minikube)에서 정상 작동
- **SC-006**: 사용자가 설정값을 변경하고 재배포할 때 기존 데이터나 세션 유실 없이 업그레이드 완료

## Assumptions

- Kubernetes 클러스터는 이미 사용자가 준비된 상태이며, 클러스터 프로비저닝은 이 기능의 범위 밖입니다
- 사용자는 kubectl, helm, terraform, pulumi 등의 CLI 도구를 각자의 환경에 미리 설치했습니다
- 컨테이너 이미지는 GitHub Container Registry (ghcr.io/egoavara/realtime-svg)에 게시되어 있으며, 기본 이미지 태그는 v0.1.4입니다
- 로드 밸런서는 클러스터 환경이 지원하는 것으로 가정합니다(클라우드 환경의 경우 자동 프로비저닝)
- TLS 인증서는 cert-manager 또는 외부 제공 방식으로 관리되며, 이 패키지는 기본 HTTP 설정을 제공합니다
- 모니터링과 로깅은 클러스터 수준에서 별도로 설정되어 있으며, 이 패키지는 표준 로그 출력만 제공합니다
- Redis는 외부 관리형 서비스 또는 클러스터 내 인메모리 방식으로 배포되며, 클러스터 내 Redis는 개발/테스트 용도로 적합합니다 (프로덕션에서는 외부 관리형 Redis 권장)
- 클러스터 내 Redis를 사용할 경우 Pod 재시작 시 데이터가 유실되므로 영속성이 필요한 프로덕션 환경에서는 외부 Redis 사용을 권장합니다

## Dependencies

- Kubernetes 클러스터 (버전 1.24 이상 권장)
- 컨테이너 레지스트리 접근 (Docker Hub 또는 사용자 지정)
- Redis 인스턴스 (외부 또는 클러스터 내)
- 배포 도구: Helm 3.x, kubectl 1.24+, Terraform 1.0+ (옵션), Pulumi 3.x+ (옵션)

## Scope Boundaries

### In Scope
- Helm, kubectl, Terraform, Pulumi를 위한 배포 패키지 제공
- 기본 설정값과 커스터마이징 가능한 파라미터 정의
- 배포 문서 및 예제 제공
- 헬스 체크 및 기본 리소스 제한 설정

### Out of Scope
- Kubernetes 클러스터 프로비저닝
- CI/CD 파이프라인 구성
- 자동 스케일링 정책 (HPA) 설정
- 고급 모니터링 및 알림 시스템
- 백업 및 재해 복구 솔루션
- 멀티 클러스터 또는 멀티 리전 배포
- 프로덕션 레벨의 보안 강화 (네트워크 정책, Pod Security Standards 등)
