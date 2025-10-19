# Pulumi Deployment Guide

이 가이드는 Pulumi를 사용하여 realtime-svg를 Kubernetes에 배포하는 방법을 설명합니다.

## 사전 요구사항

- Kubernetes 클러스터 (1.24+)
- Pulumi CLI 3.0+ 설치
- Node.js 18+ 설치
- kubectl 설치 및 클러스터 접근 설정
- Pulumi 계정 (pulumi.com) 또는 로컬 백엔드 사용

## Pulumi 설치

### Pulumi CLI 설치

```bash
# macOS
brew install pulumi/tap/pulumi

# Linux
curl -fsSL https://get.pulumi.com | sh

# Windows
choco install pulumi

# 설치 확인
pulumi version
```

### Pulumi 로그인

```bash
# Pulumi Cloud 사용 (무료)
pulumi login

# 로컬 백엔드 사용
pulumi login --local
```

## 빠른 시작

### 1. 저장소 클론

```bash
git clone https://github.com/egoavara/realtime-svg.git
cd realtime-svg/deploy/pulumi
```

### 2. 종속성 설치

```bash
npm install
```

### 3. 스택 생성

```bash
# dev 스택 생성
pulumi stack init dev

# 또는 기존 스택 선택
pulumi stack select dev
```

### 4. 설정 구성

```bash
# 기본 설정
pulumi config set namespace default
pulumi config set replicas 2
pulumi config set imageTag v0.1.4

# Ingress 설정
pulumi config set ingressEnabled true
pulumi config set ingressHost realtime-svg.example.com

# Redis 설정 (in-cluster)
pulumi config set redisEnabled true
```

### 5. 미리보기 및 배포

```bash
# 변경사항 미리보기
pulumi preview

# 배포 실행
pulumi up

# 자동 승인
pulumi up --yes
```

### 6. 배포 확인

```bash
# 출력 값 확인
pulumi stack output

# Kubernetes 리소스 확인
kubectl get all -n default -l app.kubernetes.io/name=realtime-svg
```

## 배포 시나리오

### 개발 환경 (in-cluster Redis)

```bash
pulumi stack init dev

pulumi config set namespace dev
pulumi config set replicas 1
pulumi config set redisEnabled true
pulumi config set ingressEnabled false
pulumi config set serviceType NodePort

pulumi up
```

### 스테이징 환경 (외부 Redis)

```bash
pulumi stack init staging

pulumi config set namespace staging
pulumi config set replicas 2
pulumi config set redisEnabled false
pulumi config set redisExternalUrl redis://redis.staging.svc:6379/ --secret
pulumi config set ingressEnabled true
pulumi config set ingressHost realtime-svg-staging.example.com

pulumi up
```

### 프로덕션 환경 (외부 Redis + TLS)

```bash
pulumi stack init production

pulumi config set namespace production
pulumi config set replicas 5

# 리소스 제한
pulumi config set resourcesRequestsCpu 500m
pulumi config set resourcesRequestsMemory 512Mi
pulumi config set resourcesLimitsCpu 2000m
pulumi config set resourcesLimitsMemory 2Gi

# 외부 Redis
pulumi config set redisEnabled false
pulumi config set redisExternalUrl redis://:password@redis.prod.svc:6379/ --secret

# Ingress + TLS
pulumi config set ingressEnabled true
pulumi config set ingressHost realtime-svg.mycompany.com
pulumi config set ingressTlsEnabled true
pulumi config set ingressTlsSecretName realtime-svg-tls

pulumi up
```

## 설정 관리

### 설정 확인

```bash
# 모든 설정 보기
pulumi config

# 특정 설정 보기
pulumi config get namespace
```

### 설정 변경

```bash
# 일반 설정
pulumi config set replicas 3

# 비밀 설정 (암호화됨)
pulumi config set redisPassword mypassword --secret

# 설정 제거
pulumi config rm redisPassword
```

### 설정 파일 직접 편집

`Pulumi.dev.yaml` 파일을 직접 편집할 수도 있습니다:

```yaml
config:
  realtime-svg:namespace: production
  realtime-svg:replicas: 5
  realtime-svg:imageTag: v0.1.5
```

## 주요 설정 옵션

| 설정 | 기본값 | 설명 |
|------|--------|------|
| `namespace` | `default` | Kubernetes 네임스페이스 |
| `replicas` | `2` | Pod 복제본 수 |
| `imageRepository` | `ghcr.io/egoavara/realtime-svg` | 이미지 저장소 |
| `imageTag` | `v0.1.4` | 이미지 태그 |
| `redisEnabled` | `true` | 클러스터 내 Redis 배포 |
| `redisExternalUrl` | `""` | 외부 Redis URL |
| `ingressEnabled` | `true` | Ingress 생성 |
| `ingressHost` | `realtime-svg.example.com` | Ingress 호스트명 |
| `serviceType` | `ClusterIP` | Service 타입 |
| `servicePort` | `80` | Service 포트 |

전체 설정 목록은 [configuration-reference.md](./configuration-reference.md)를 참조하세요.

## 업데이트

### 애플리케이션 버전 업데이트

```bash
pulumi config set imageTag v0.1.5
pulumi up
```

### 복제본 수 변경

```bash
pulumi config set replicas 5
pulumi up
```

### 외부 Redis로 전환

```bash
pulumi config set redisEnabled false
pulumi config set redisExternalUrl redis://redis.external.svc:6379/ --secret
pulumi up
```

## 스택 관리

### 여러 스택 사용

```bash
# 스택 목록
pulumi stack ls

# 새 스택 생성
pulumi stack init production

# 스택 전환
pulumi stack select dev

# 스택 제거
pulumi stack rm dev
```

### 스택 간 설정 복사

```bash
# dev 스택의 설정을 production으로 복사
pulumi stack select dev
pulumi config -j > config.json

pulumi stack select production
cat config.json | pulumi config set-all -f -
```

## 출력 값

배포 후 다음 출력 값을 확인할 수 있습니다:

```bash
pulumi stack output

# 특정 출력 값
pulumi stack output serviceName
pulumi stack output ingressUrl
```

출력 값:
- `deploymentName`: Deployment 이름
- `serviceName`: Service 이름
- `serviceType`: Service 타입
- `ingressUrl`: Ingress URL (활성화된 경우)
- `redisServiceName`: Redis Service 이름 (redis_enabled=true)

## 제거

### 리소스 제거

```bash
# 미리보기
pulumi destroy --preview

# 제거 실행
pulumi destroy

# 자동 승인
pulumi destroy --yes
```

### 스택 및 설정 제거

```bash
pulumi stack rm dev
```

## 상태 관리

### Pulumi Cloud 백엔드 (기본)

```bash
pulumi login
```

- 자동 상태 관리 및 동기화
- 웹 UI에서 배포 이력 확인
- 팀 협업 기능

### 로컬 백엔드

```bash
pulumi login --local

# 로컬 파일 시스템에 상태 저장 (~/.pulumi)
```

### S3 백엔드

```bash
pulumi login s3://my-pulumi-state-bucket
```

### 상태 확인

```bash
# 현재 스택 상태
pulumi stack

# 리소스 목록
pulumi stack export
```

## 프로그래밍 방식 확장

Pulumi는 TypeScript로 작성되어 있어 프로그래밍 방식으로 확장 가능합니다:

```typescript
// Custom validation
if (replicas > 10) {
    throw new Error("Replicas should not exceed 10");
}

// Computed values
const appUrl = pulumi.interpolate`https://${ingressHost}/app`;

// Conditional resources
const monitoring = enableMonitoring 
    ? new k8s.apps.v1.Deployment("monitoring", {...})
    : undefined;
```

## CI/CD 통합

### GitHub Actions

```yaml
name: Deploy with Pulumi

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: pulumi/actions@v4
        with:
          command: up
          stack-name: production
        env:
          PULUMI_ACCESS_TOKEN: ${{ secrets.PULUMI_ACCESS_TOKEN }}
```

### GitLab CI

```yaml
deploy:
  image: pulumi/pulumi-nodejs
  script:
    - pulumi login
    - pulumi stack select production
    - pulumi up --yes
  environment:
    name: production
```

## 트러블슈팅

### TypeScript 컴파일 오류

```bash
# 종속성 재설치
rm -rf node_modules package-lock.json
npm install

# TypeScript 검증
npm run build
```

### Provider 인증 오류

```bash
# kubeconfig 확인
kubectl cluster-info

# Pulumi에서 kubeconfig 경로 설정
pulumi config set kubernetes:kubeconfig ~/.kube/config
```

### 상태 동기화 오류

```bash
# 상태 새로고침
pulumi refresh

# 강제 새로고침
pulumi refresh --yes
```

### 리소스 임포트

기존 Kubernetes 리소스를 Pulumi로 가져오기:

```bash
pulumi import kubernetes:core/v1:Service my-service default/realtime-svg
```

## 모범 사례

### 1. 스택 분리

```bash
# 환경별 스택 생성
pulumi stack init dev
pulumi stack init staging
pulumi stack init production
```

### 2. 비밀 관리

```bash
# 항상 --secret 플래그 사용
pulumi config set redisPassword mypass --secret
pulumi config set apiKey secret123 --secret
```

### 3. 버전 관리

```bash
# 스택 설정 파일을 Git에 커밋
git add Pulumi.*.yaml
git commit -m "Update production configuration"

# 비밀 값은 암호화되어 안전함
```

### 4. 자동화

```bash
# 자동 승인 사용 (CI/CD)
pulumi up --yes --skip-preview
```

### 5. 리소스 태그

```typescript
const labels = {
    "app.kubernetes.io/name": appName,
    environment: pulumi.getStack(),
    team: "platform",
};
```

## 참고 자료

- [Pulumi Kubernetes Provider 문서](https://www.pulumi.com/registry/packages/kubernetes/)
- [Pulumi Best Practices](https://www.pulumi.com/docs/using-pulumi/best-practices/)
- [Configuration Reference](./configuration-reference.md)
- [Architecture Overview](./architecture.md)
