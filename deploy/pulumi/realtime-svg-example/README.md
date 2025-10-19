# realtime-svg 배포 예시

`realtime-svg-pulumi` 모듈을 사용하여 Kubernetes 클러스터에 배포하는 예시입니다.

## 사용 방법

```bash
cd deploy/pulumi/realtime-svg-example

# 종속성 설치 (로컬 모듈 링크됨)
npm install

# 스택 생성
pulumi stack init dev

# 현재 kubectl 컨텍스트 확인
kubectl config current-context

# Kubernetes 컨텍스트 설정
pulumi config set kubeContext $(kubectl config current-context)

# 배포
pulumi up

# 결과 확인
kubectl get all -n default -l app.kubernetes.io/name=realtime-svg
```

## 모듈 사용

이 예시는 `file:../realtime-svg`로 로컬 모듈을 참조합니다:

```json
{
  "dependencies": {
    "realtime-svg-pulumi": "file:../realtime-svg"
  }
}
```

```typescript
import * as pulumi from "@pulumi/pulumi";
import * as k8s from "@pulumi/kubernetes";
import { RealtimeSvg } from "realtime-svg-pulumi";

const config = new pulumi.Config();
const kubeContext = config.require("kubeContext");

const provider = new k8s.Provider("k8s", {
    context: kubeContext,
});

const app = new RealtimeSvg("realtime-svg", {
    provider: provider,
    namespace: "default",
    replicas: 2,
    imageTag: "v0.1.4",
});
```

## 배포 내용

- **네임스페이스**: default
- **복제본**: 2
- **이미지**: ghcr.io/egoavara/realtime-svg:v0.1.4
- **Redis**: 활성화 (in-cluster)
- **Ingress**: 비활성화
- **Service**: ClusterIP, 포트 80

## 설정 커스터마이징

### 현재 kubectl 컨텍스트 사용

```bash
pulumi config set kubeContext $(kubectl config current-context)
```

### 특정 컨텍스트 지정

```bash
# 사용 가능한 컨텍스트 확인
kubectl config get-contexts

# 특정 컨텍스트 설정
pulumi config set kubeContext your-context-name
```

### Pulumi.dev.yaml 직접 편집

```yaml
config:
  realtime-svg-example:kubeContext: "your-context-name"
```

## 제거

```bash
pulumi destroy
```
