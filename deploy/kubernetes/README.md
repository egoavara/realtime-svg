# kubectl 배포 (테스트용)

단일 YAML 파일로 realtime-svg를 Kubernetes에 배포하기 위한 테스트 전용 패키지입니다.

## 빠른 배포

```bash
# 현재 컨텍스트 확인
kubectl config current-context

# 배포
kubectl apply -f deploy/kubernetes/all-in-one.yaml

# 확인
kubectl get all -n realtime-svg

# 포트 포워딩으로 테스트
kubectl port-forward -n realtime-svg service/realtime-svg 8080:80

# 브라우저에서 http://localhost:8080 접속
```

## 배포되는 리소스

- **Namespace**: `realtime-svg`
- **ConfigMap**: `realtime-svg-config` (환경 변수)
- **Secret**: `realtime-svg-secret` (빈 시크릿, 필요시 데이터 추가)
- **Deployment**: `realtime-svg` (애플리케이션, 2 replicas)
- **Deployment**: `realtime-svg-redis` (Redis, 1 replica)
- **Service**: `realtime-svg` (ClusterIP, 포트 80)
- **Service**: `realtime-svg-redis` (ClusterIP, 포트 6379)

## 설정

이 패키지는 **테스트 전용**이며 매개변수화되어 있지 않습니다. 프로덕션 환경에서는 다음 방법을 사용하세요:

- **Helm**: `deploy/helm/realtime-svg/` - 매개변수화, 재사용 가능
- **Terraform**: `deploy/terraform/realtime-svg/` - IaC
- **Pulumi**: `deploy/pulumi/realtime-svg/` - 프로그래밍 방식 IaC

## 기본 설정

- **이미지**: `ghcr.io/egoavara/realtime-svg:v0.1.4`
- **복제본**: 2 (애플리케이션), 1 (Redis)
- **Redis**: 클러스터 내 배포 (인메모리, 영속성 없음)
- **리소스 요청**: CPU 100m, 메모리 128Mi
- **리소스 제한**: CPU 500m, 메모리 512Mi (앱) / 200m, 256Mi (Redis)

## 제거

```bash
kubectl delete -f deploy/kubernetes/all-in-one.yaml
```

또는 네임스페이스 전체 삭제:

```bash
kubectl delete namespace realtime-svg
```

## 주의사항

⚠️ **테스트 전용**: 이 패키지는 빠른 테스트를 위한 것이며, 프로덕션 환경에서는 Helm/Terraform/Pulumi를 사용하세요.

- Redis 데이터는 영속화되지 않습니다 (Pod 재시작 시 초기화)
- Ingress가 포함되어 있지 않습니다 (외부 접근 시 `kubectl port-forward` 사용)
- 설정 변경 시 YAML 파일을 직접 수정해야 합니다
