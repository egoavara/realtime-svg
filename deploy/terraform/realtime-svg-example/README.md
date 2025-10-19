# realtime-svg Terraform 배포 예시

클러스터에 배포하는 간단한 예시입니다.

## 사용 방법

```bash
kubectl config current-context
```

위 명령어를 입력해 현재 컨텍스트 이름 획득

이후 `deploy/terraform/realtime-svg-example/main.tf` 의 `{{ kube_context 이름 }}` 필드를 위 이름으로 변경

이후 아래 명령어 실행


```bash

cd deploy/terraform/realtime-svg-example

# 초기화
terraform init

# 계획 확인
terraform plan

# 배포 실행
terraform apply

# 결과 확인
kubectl get all -n default -l app.kubernetes.io/name=realtime-svg
```

## 배포 내용

- **네임스페이스**: default
- **복제본 수**: 2
- **이미지**: ghcr.io/egoavara/realtime-svg:v0.1.4
- **Redis**: 클러스터 내부 (in-cluster)
- **Ingress**: 활성화 (realtime-svg.egoavara.net)
- **Service**: ClusterIP, 포트 80

## 제거

```bash
terraform destroy
```
