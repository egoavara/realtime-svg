# realtime-svg 설계 메모 (multipart/x-mixed-replace)

연제나 한국어로 대답한다

## 프로젝트 목표
- `multipart/x-mixed-replace` 스트리밍 응답을 활용해 SVG 이미지를 실시간으로 갱신한다.
- Tera 템플릿 엔진을 사용해 SVG 내 동적 콘텐츠(텍스트, 속성 등)를 파라미터로 치환한다.
- 서버 측 파라미터 변경 이벤트마다 템플릿을 다시 렌더링하여 모든 연결된 클라이언트에 실시간으로 브로드캐스트한다.
- Rust 기반 멀티 크레이트(workspace) 구조를 유지하면서, 공통 로직과 프런트엔드/백엔드 역할을 명확히 분리한다.

## 핵심 아키텍처
- **템플릿 렌더링**: Tera 템플릿 엔진으로 SVG 생성 (`{{ variable }}` 구문 사용)
- **세션 관리**: Redis에 세션 데이터(template + args) 저장, TTL 지원
- **실시간 브로드캐스트**: Redis pubsub으로 파라미터 업데이트 이벤트 전파
- **스트리밍 프로토콜**: `multipart/x-mixed-replace; boundary=frame` 헤더로 프레임 단위 전송
## 자동 버전 업데이트 시스템 (AUTOUPDATE)

### 개요
`prepare-release` 워크플로우는 릴리즈 시 모든 배포 모듈의 버전을 자동으로 업데이트합니다.
실수로 잘못된 위치를 변경하는 것을 방지하기 위해 **AUTOUPDATE 마커 시스템**을 사용합니다.

### AUTOUPDATE 마커 규칙

#### 절대 규칙
1. **AUTOUPDATE 주석이 있는 라인은 절대 수정/삭제하지 마세요**
2. **AUTOUPDATE 주석 바로 다음 줄의 버전 값은 수동으로 편집하지 마세요**
3. 새로운 버전 관련 필드를 추가할 때는 AUTOUPDATE 마커를 추가하지 마세요 (워크플로우가 업데이트하지 않는 값)

#### 마커 위치 및 형식

**Helm values.yaml**
```yaml
image:
  repository: ghcr.io/egoavara/realtime-svg
  # AUTOUPDATE: prepare-release workflow updates this version - do not edit manually
  tag: v0.1.9  # <- 이 값이 자동 업데이트됨
```

**Pulumi index.ts**
```typescript
const imageRepository = args.imageRepository || "ghcr.io/egoavara/realtime-svg";
// AUTOUPDATE: prepare-release workflow updates this version - do not edit manually
const imageTag = args.imageTag || "v0.1.9";  // <- 이 값이 자동 업데이트됨
```

**Kubernetes install.yaml**
```yaml
containers:
- name: realtime-svg
  # AUTOUPDATE: prepare-release workflow updates this version - do not edit manually
  image: ghcr.io/egoavara/realtime-svg:v0.1.9  # <- 이 값이 자동 업데이트됨
```

**Terraform variables.tf**
```hcl
# AUTOUPDATE: prepare-release workflow updates this version - do not edit manually
variable "image_tag" {
  description = "Container image tag"
  type        = string
  default     = "v0.1.9"  # <- 이 값이 자동 업데이트됨
}
```

### 자동 업데이트 대상 파일

| 파일 | 업데이트 항목 | 방법 |
|------|--------------|------|
| `Cargo.toml` | `version = "VERSION"` | sed (마커 없음) |
| `deploy/helm/realtime-svg/Chart.yaml` | `version:`, `appVersion:` | sed (마커 없음) |
| `deploy/helm/realtime-svg/values.yaml` | `.image.tag` | sed + AUTOUPDATE 마커 |
| `deploy/pulumi/realtime-svg/package.json` | `"version": "VERSION"` | sed (마커 없음) |
| `deploy/pulumi/realtime-svg/index.ts` | `const imageTag = ...` | sed + AUTOUPDATE 마커 |
| `deploy/kubernetes/realtime-svg/install.yaml` | `image: ghcr.io/.../realtime-svg:vVERSION` | sed + AUTOUPDATE 마커 |
| `deploy/terraform/realtime-svg/variables.tf` | `variable "image_tag" { default = ... }` | sed + AUTOUPDATE 마커 |

### Redis 버전 고정
모든 배포 모듈에서 Redis 버전은 `8-alpine`으로 **고정**되어 있습니다.
prepare-release 워크플로우는 Redis 버전을 변경하지 않습니다.

### 워크플로우 작동 원리

```bash
# AUTOUPDATE 마커가 있는 다음 줄만 정확하게 업데이트
sed -i '/# AUTOUPDATE.*prepare-release/{ n; s/tag: v.*/tag: vNEW_VERSION/; }' values.yaml
```

이 방식으로:
- ✅ AUTOUPDATE 마커가 있는 위치만 업데이트
- ✅ 다른 곳에 유사한 패턴이 있어도 안전
- ✅ 개발자가 실수로 코드를 수정해도 워크플로우는 정확한 위치만 업데이트

### 개발 시 주의사항

1. **버전 관련 코드 수정 시**
   - AUTOUPDATE 마커가 있는지 먼저 확인
   - 마커가 있다면 그 값은 자동 관리되므로 수동 수정 불필요
   - 마커를 삭제하거나 이동하지 마세요

2. **새로운 배포 모듈 추가 시**
   - 이미지 버전이 하드코딩되어야 한다면 AUTOUPDATE 마커 추가
   - `.github/workflows/prepare-release.yml`에 업데이트 단계 추가
   - 마커 기반 sed 명령 작성

3. **리팩토링 시**
   - AUTOUPDATE 마커 위치 유지
   - 코드 구조 변경 시 워크플로우 sed 명령도 함께 업데이트

