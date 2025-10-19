# realtime-svg Pulumi Program

Pulumi와 TypeScript를 사용하여 realtime-svg 애플리케이션을 Kubernetes에 배포하는 프로그램입니다.

## 특징

- ✅ TypeScript로 작성된 타입 안전 IaC
- ✅ 프로그래밍 방식의 인프라 정의
- ✅ 자동 상태 관리 (Pulumi Cloud)
- ✅ 조건부 리소스 생성 (Redis, Ingress)
- ✅ 스택별 설정 관리
- ✅ 일관된 매개변수 구조 (Helm, Terraform, kubectl과 동일)

## 사전 요구사항

- **Pulumi CLI**: 3.0 이상
- **Node.js**: 18 이상
- **Kubernetes**: 1.24 이상
- **kubectl**: 클러스터 접근 설정 완료

## 빠른 시작

### 1. 종속성 설치

```bash
cd deploy/pulumi
npm install
```

### 2. Pulumi 로그인

```bash
# Pulumi Cloud (무료)
pulumi login

# 또는 로컬 백엔드
pulumi login --local
```

### 3. 스택 초기화

```bash
pulumi stack init dev
```

### 4. 설정 구성

```bash
pulumi config set namespace default
pulumi config set ingressHost realtime-svg.example.com
```

### 5. 배포

```bash
pulumi up
```

## 주요 파일

| 파일 | 설명 |
|------|------|
| `index.ts` | 메인 Pulumi 프로그램 (모든 리소스 정의) |
| `Pulumi.yaml` | 프로젝트 메타데이터 |
| `Pulumi.dev.yaml` | dev 스택 설정 (예시) |
| `package.json` | Node.js 종속성 |
| `tsconfig.json` | TypeScript 설정 |

## 배포 예제

### 개발 환경

```bash
pulumi stack init dev

pulumi config set namespace dev
pulumi config set replicas 1
pulumi config set ingressEnabled false
pulumi config set serviceType NodePort

pulumi up
```

### 프로덕션 환경

```bash
pulumi stack init production

pulumi config set namespace production
pulumi config set replicas 5
pulumi config set redisEnabled false
pulumi config set redisExternalUrl redis://:password@redis.prod.svc:6379/ --secret
pulumi config set ingressEnabled true
pulumi config set ingressHost realtime-svg.mycompany.com
pulumi config set ingressTlsEnabled true
pulumi config set ingressTlsSecretName realtime-svg-tls

pulumi up
```

## 설정 옵션

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

전체 설정 목록은 `Pulumi.dev.yaml` 파일을 참조하세요.

## 출력 값

```bash
pulumi stack output
```

- `deploymentName`: Deployment 이름
- `serviceName`: Service 이름
- `serviceType`: Service 타입
- `ingressUrl`: Ingress URL (활성화된 경우)
- `redisServiceName`: Redis Service 이름

## 일반적인 작업

### 설정 변경

```bash
pulumi config set replicas 3
pulumi up
```

### 버전 업데이트

```bash
pulumi config set imageTag v0.1.5
pulumi up
```

### 외부 Redis 사용

```bash
pulumi config set redisEnabled false
pulumi config set redisExternalUrl redis://redis.external.svc:6379/ --secret
pulumi up
```

### 미리보기

```bash
pulumi preview
```

### 제거

```bash
pulumi destroy
```

## 스택 관리

```bash
# 스택 목록
pulumi stack ls

# 스택 전환
pulumi stack select production

# 새 스택 생성
pulumi stack init staging

# 스택 제거
pulumi stack rm dev
```

## TypeScript 컴파일

```bash
# TypeScript 빌드
npm run build

# TypeScript 검증 (컴파일 없이)
npx tsc --noEmit
```

## 프로그래밍 방식 확장

`index.ts`를 편집하여 커스텀 로직 추가:

```typescript
// 조건부 검증
if (replicas > 10) {
    throw new Error("Too many replicas!");
}

// 계산된 값
const appUrl = pulumi.interpolate`https://${ingressHost}/app`;

// 반복문으로 리소스 생성
for (let i = 0; i < workerCount; i++) {
    new k8s.apps.v1.Deployment(`worker-${i}`, {...});
}
```

## CI/CD 통합

### GitHub Actions

```yaml
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
    - pulumi up --yes
```

## 트러블슈팅

### Provider 인증

```bash
pulumi config set kubernetes:kubeconfig ~/.kube/config
```

### 상태 새로고침

```bash
pulumi refresh
```

### 종속성 재설치

```bash
rm -rf node_modules package-lock.json
npm install
```

## 상태 백엔드

### Pulumi Cloud (기본)

```bash
pulumi login
```

자동 상태 관리, 웹 UI, 팀 협업 기능 제공

### 로컬 백엔드

```bash
pulumi login --local
```

`~/.pulumi`에 로컬 저장

### S3 백엔드

```bash
pulumi login s3://my-bucket
```

## 문서

- [Pulumi 배포 가이드](../../docs/deployment/pulumi-guide.md)
- [설정 레퍼런스](../../docs/deployment/configuration-reference.md)
- [아키텍처 문서](../../docs/deployment/architecture.md)

## 참고 자료

- [Pulumi Kubernetes Provider](https://www.pulumi.com/registry/packages/kubernetes/)
- [Pulumi TypeScript SDK](https://www.pulumi.com/docs/languages-sdks/javascript/)
- [Pulumi Best Practices](https://www.pulumi.com/docs/using-pulumi/best-practices/)

## 지원

- GitHub Issues: https://github.com/egoavara/realtime-svg/issues
- Documentation: https://github.com/egoavara/realtime-svg/tree/main/docs
