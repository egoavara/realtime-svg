# Quickstart: Kubernetes 배포 패키징

**Feature**: 007-k8s-deployment-packaging  
**Updated**: 2025-10-19

## 5분 안에 시작하기

### 전제 조건

- Kubernetes 클러스터 (minikube, kind, GKE, EKS, AKS 등)
- `kubectl` 설치 및 클러스터 접근 설정
- 선택한 배포 도구 설치 (아래 중 하나)
  - Helm 3.x
  - Terraform 1.0+
  - Pulumi 3.x+

### Option 1: Helm으로 배포 (권장)

```bash
# 1. Helm 차트 다운로드 (릴리스에서)
wget https://github.com/username/realtime-svg/releases/latest/download/realtime-svg-helm-latest.tgz
tar -xzf realtime-svg-helm-latest.tgz

# 또는 Git 리포지토리에서 직접
git clone https://github.com/username/realtime-svg.git
cd realtime-svg

# 2. 설정 커스터마이징 (선택)
cat > custom-values.yaml <<EOF
namespace: default
image:
  repository: ghcr.io/egoavara/realtime-svg
  tag: v0.1.4
ingress:
  host: realtime-svg.example.com
EOF

# 3. 설치
helm install realtime-svg ./deploy/helm/realtime-svg -f custom-values.yaml

# 4. 상태 확인
kubectl get pods
kubectl get svc
kubectl get ingress

# 5. 접속 테스트
# Ingress 사용 시
curl http://realtime-svg.example.com

# 또는 포트 포워딩
kubectl port-forward svc/frontend 8080:80
# 브라우저에서 http://localhost:8080 접속
```

**예상 시간**: 3분

---

### Option 2: kubectl로 배포

```bash
# 1. 매니페스트 다운로드
git clone https://github.com/username/realtime-svg.git
cd realtime-svg/deploy/kubernetes

# 2. 이미지 태그 수정 (필요 시)
sed -i 's|ghcr.io/egoavara/realtime-svg:v0.1.4|ghcr.io/egoavara/realtime-svg:v0.2.0|' deployment.yaml

# 3. Secret 생성 (필요 시)
cp secret.yaml.example secret.yaml
# secret.yaml 편집하여 base64 인코딩된 값 입력

# 4. 배포
kubectl apply -f .

# 5. 상태 확인
kubectl get all

# 6. 접속 테스트
kubectl port-forward svc/frontend 8080:80
```

**예상 시간**: 5분

---

### Option 3: Terraform으로 배포

```bash
# 1. Terraform 설정 다운로드
git clone https://github.com/username/realtime-svg.git
cd realtime-svg/deploy/terraform

# 2. 변수 설정
cat > terraform.tfvars <<EOF
namespace = "default"
image_repository = "ghcr.io/egoavara/realtime-svg"
image_tag = "v0.1.4"
ingress_host = "realtime-svg.example.com"
EOF

# 3. 초기화 및 배포
terraform init
terraform plan
terraform apply

# 4. 출력 확인
terraform output

# 5. 접속 테스트
kubectl port-forward svc/frontend 8080:80
```

**예상 시간**: 5분

---

### Option 4: Pulumi로 배포

```bash
# 1. Pulumi 프로그램 다운로드
git clone https://github.com/username/realtime-svg.git
cd realtime-svg/deploy/pulumi

# 2. 의존성 설치
npm install

# 3. Pulumi 스택 생성
pulumi stack init dev

# 4. 설정
pulumi config set namespace default
pulumi config set image:repository ghcr.io/egoavara/realtime-svg
pulumi config set image:tag v0.1.4
pulumi config set ingress:host realtime-svg.example.com

# 5. 배포
pulumi up

# 6. 접속 테스트
kubectl port-forward svc/frontend 8080:80
```

**예상 시간**: 7분

---

## 일반적인 설정 시나리오

### 시나리오 1: 외부 Redis 사용

```yaml
# Helm values.yaml
redis:
  enabled: false
  host: my-redis.example.com
  port: 6379
  password: my-secret-password
```

```hcl
# Terraform terraform.tfvars
redis_enabled = false
redis_host = "my-redis.example.com"
redis_password = "my-secret-password"
```

---

### 시나리오 2: TLS/HTTPS 활성화

```yaml
# Helm values.yaml
ingress:
  enabled: true
  host: realtime-svg.example.com
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  tls:
    enabled: true
    secretName: realtime-svg-tls
```

사전 조건: cert-manager 설치 및 ClusterIssuer 설정

---

### 시나리오 3: 프로덕션 리소스 설정

```yaml
# Helm values.yaml
replicas: 5

resources:
  requests:
    cpu: 500m
    memory: 512Mi
  limits:
    cpu: 2000m
    memory: 2Gi
```

---

### 시나리오 4: NodePort로 노출

```yaml
# Helm values.yaml
ingress:
  enabled: false

service:
  type: NodePort
```

접속:
```bash
# NodePort 확인
kubectl get svc realtime-svg -o jsonpath='{.spec.ports[0].nodePort}'

# Minikube 사용 시
minikube service realtime-svg
```

---

## 문제 해결

### Pod가 Pending 상태

```bash
# 원인 확인
kubectl describe pod <pod-name>

# 일반적인 원인:
# 1. 이미지 pull 실패 → imagePullSecrets 확인
# 2. 리소스 부족 → 클러스터 용량 확인
# 3. 스케줄링 제약 → nodeSelector/affinity 확인
```

### Ingress가 작동하지 않음

```bash
# Ingress 컨트롤러 설치 확인
kubectl get pods -n ingress-nginx

# Ingress 컨트롤러 설치 (minikube)
minikube addons enable ingress

# Ingress 리소스 확인
kubectl describe ingress
```

### Redis 연결 오류

```bash
# ConfigMap 확인
kubectl get configmap -o yaml

# Secret 확인
kubectl get secret -o yaml

# Backend 로그 확인
kubectl logs deployment/backend
```

---

## 제거

### Helm
```bash
helm uninstall realtime-svg
```

### kubectl
```bash
kubectl delete -f deploy/kubernetes/
```

### Terraform
```bash
terraform destroy
```

### Pulumi
```bash
pulumi destroy
```

---

## 다음 단계

1. **설정 커스터마이징**: [configuration-reference.md](../docs/deployment/configuration-reference.md) 참조
2. **프로덕션 배포**: 각 도구별 가이드 참조
   - [Helm 가이드](../docs/deployment/helm-guide.md)
   - [kubectl 가이드](../docs/deployment/kubectl-guide.md)
   - [Terraform 가이드](../docs/deployment/terraform-guide.md)
   - [Pulumi 가이드](../docs/deployment/pulumi-guide.md)
3. **모니터링 설정**: Prometheus, Grafana 통합
4. **CI/CD 구성**: GitHub Actions, GitLab CI 등

---

## 지원

- 이슈: https://github.com/username/realtime-svg/issues
- 문서: https://username.github.io/realtime-svg/deployment/
- 예제: https://github.com/username/realtime-svg/tree/main/examples
