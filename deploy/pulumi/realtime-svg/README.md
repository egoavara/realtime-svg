# realtime-svg Pulumi Module

재사용 가능한 Pulumi TypeScript 모듈입니다.

## 사용 방법

### 로컬 모듈로 사용

```json
{
  "dependencies": {
    "realtime-svg-pulumi": "file:../path/to/realtime-svg"
  }
}
```

```typescript
import * as pulumi from "@pulumi/pulumi";
import * as k8s from "@pulumi/kubernetes";
import { RealtimeSvg } from "realtime-svg-pulumi";

const provider = new k8s.Provider("k8s", {
    context: "admin@kubernetes-egoavara-net",
});

const app = new RealtimeSvg("my-app", {
    provider: provider,
    namespace: "default",
    replicas: 2,
    imageTag: "v0.1.4",
    redisEnabled: true,
    ingressEnabled: true,
    ingressHost: "realtime-svg.egoavara.net",
    servicePort: 80,
});

export const deploymentName = app.deploymentName;
export const serviceName = app.serviceName;
export const ingressUrl = app.ingressUrl;
```

## API

### RealtimeSvg 클래스

```typescript
class RealtimeSvg extends ComponentResource
```

### 생성자

```typescript
new RealtimeSvg(name: string, args: RealtimeSvgArgs, opts?: ComponentResourceOptions)
```

### RealtimeSvgArgs

| 속성 | 타입 | 기본값 | 설명 |
|------|------|--------|------|
| `provider` | `k8s.Provider` | - | Kubernetes provider |
| `namespace` | `string` | `"default"` | 네임스페이스 |
| `replicas` | `number` | `2` | Pod 복제본 수 |
| `imageRepository` | `string` | `"ghcr.io/egoavara/realtime-svg"` | 이미지 저장소 |
| `imageTag` | `string` | `"v0.1.4"` | 이미지 태그 |
| `redisEnabled` | `boolean` | `true` | Redis 배포 여부 |
| `redisExternalUrl` | `string` | `""` | 외부 Redis URL |
| `ingressEnabled` | `boolean` | `true` | Ingress 생성 여부 |
| `ingressHost` | `string` | `"realtime-svg.example.com"` | Ingress 호스트 |
| `serviceType` | `string` | `"ClusterIP"` | Service 타입 |
| `servicePort` | `number` | `80` | Service 포트 |

### 출력

| 속성 | 타입 | 설명 |
|------|------|------|
| `deploymentName` | `Output<string>` | Deployment 이름 |
| `serviceName` | `Output<string>` | Service 이름 |
| `serviceType` | `Output<string>` | Service 타입 |
| `ingressUrl` | `Output<string>` | Ingress URL |
| `redisServiceName` | `Output<string>` | Redis Service 이름 |

## 예시

`../realtime-svg-example` 디렉토리를 참조하세요.
