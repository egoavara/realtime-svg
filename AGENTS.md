# realtime-svg 설계 메모 (multipart/x-mixed-replace)

연제나 한국어로 대답한다

## 프로젝트 목표
- `multipart/x-mixed-replace` 스트리밍 응답을 활용해 SVG 이미지를 실시간으로 갱신한다.
- SVG 내 특정 텍스트 노드 값이 서버 측 변경 이벤트마다 자동으로 덮어씌워지도록 한다.
- Rust 기반 멀티 크레이트(workspace) 구조를 유지하면서, 공통 로직과 프런트엔드/백엔드 역할을 명확히 분리한다.

## 전반 아키텍처
- `common` 크레이트: SVG 템플릿 로딩/파싱, 텍스트 업데이트 도메인 타입, 직렬화/역직렬화 로직을 담당.
- `backend` 크레이트: Axum 기반 HTTP 서버. 업데이트 API와 `multipart/x-mixed-replace` 스트림 엔드포인트를 제공하며, 브로드캐스트 채널을 통해 최신 SVG를 전달.
- `frontend` 크레이트: WASM + TypeScript-less Yew(혹은 Leptos) 기반 단일 페이지 UI. 사용자 입력을 받아 업데이트 API 호출 및 결과 미리보기 제공.

## 데이터 흐름
1. 프런트엔드 사용자가 텍스트 입력 후 `POST /api/update` 호출 (`Json<UpdateRequest>` 전송).
2. 백엔드는 `common::UpdateRequest`를 파싱해 `Renderer`에게 최신 SVG 문자열을 생성하도록 요청.
3. 렌더링된 SVG 문자열을 `watch::Sender`를 통해 브로드캐스트.
4. `GET /stream.svg`로 연결된 클라이언트는 `multipart/x-mixed-replace; boundary=frame` 응답을 통해 새 SVG를 수신.
5. 프런트엔드 `<img src="/stream.svg">` 또는 `<object data="/stream.svg">` 요소가 각 파트를 즉시 갱신해 실시간 변화를 표현.

## multipart/x-mixed-replace 세부사항
- 헤더 예시: `Content-Type: multipart/x-mixed-replace; boundary=frame`.
- 각 파트는 아래와 같은 형식을 따른다:
  ```http
  --frame
  Content-Type: image/svg+xml
  Content-Length: {{len}}

  <svg ...>...</svg>
  ```
- 첫 프레임 전송 시 클라이언트가 빈 이미지를 보는 일이 없도록 초기 SVG를 즉시 전송한다.
- 백엔드 스트리머는 `tokio_stream::wrappers::WatchStream` + `StreamBody`를 사용하거나, 커스텀 `Body` 구현으로 각 업데이트마다 파트를 이어붙인다.

## Backend 세부 설계
- 종속성: `axum`, `tokio`, `tokio-stream`, `serde`, `serde_json`, `tower-http`, `include_dir`(템플릿 포함 시), `tracing`.
- `AppState` 구성 요소:
  - `renderer: Arc<RwLock<Renderer>>` – SVG 템플릿과 현재 상태 관리.
  - `tx: watch::Sender<SvgFrame>` / `rx: watch::Receiver<SvgFrame>` – 최신 SVG 문자열 브로드캐스트 채널.
- 라우트 구성:
  - `GET /stream.svg`: `multipart/x-mixed-replace` 응답, 백프레셔 대비 타임아웃 핸들링.
  - `POST /api/update`: 단일 텍스트 필드 업데이트. 유효성 검증 후 `renderer` 갱신, `tx.send(new_frame)` 호출.
  - `GET /`: 정적 프런트엔드 HTML 전달(선택적으로 `include_str!` 또는 `StaticFile` 제공).
- 오류 처리: 유효하지 않은 타겟 ID, SVG 렌더 실패 시 `StatusCode::BAD_REQUEST`와 JSON 오류 응답.
- 테스트 전략:
  - 유닛: `Renderer::apply_updates`가 특정 노드 ID를 올바르게 교체하는지 검증.
  - 통합: `axum::Router`를 이용한 `POST /api/update` 후 `rx.changed()` 결과 검사.

## Common 크레이트 역할
- `SvgTemplate` 구조체: 원본 SVG 문자열과 업데이트 가능한 노드 ID 맵(`HashMap<String, NodePath>` 등)을 보관.
- `SvgRenderer` 트레이트 & 구현체:
  - `fn from_template_str(svg: &str) -> Result<Self>`
  - `fn render_with_updates(&mut self, updates: &HashMap<String, String>) -> Result<String>`
- XML 조작 도구: `roxmltree` + 수작업 변환 또는 `xmltree`/`quick-xml` 사용. 단순 replace라면 플레이스홀더(`{{text_id}}`) 기반 `handlebars` 적용도 고려.
- DTO 정의:
  - `UpdateRequest { target: String, value: String }`
  - `SvgFrame { content: String, timestamp: DateTime<Utc> }`
  - 에러 타입 `DomainError` → `thiserror` 활용.

## Frontend 크레이트 전략
- `yew` 선택: 러닝커브와 번들 크기를 고려해 `yew` 권장.
- 기능:
  - 입력 폼과 미리보기 영역.
  - `<img id="live-svg" src="/stream.svg">`로 스트림 구독(브라우저 기본 파서 활용).
  - 폼 제출 시 `fetch("/api/update", { method: "POST", body: JSON })` 호출.
  - 응답 성공/실패 메시지 UI.
- 빌드: `trunk` 사용해 `dist/` 산출물 생성 후 `backend`가 정적 파일 서비스.
- 개발 편의: 핫리로드 필요 시 `trunk serve` + 백엔드 프록시 구성.

## 초기 개발 순서
1. `common`에서 SVG 템플릿 로더 및 업데이트 로직 구현 (테스트 포함).
2. `backend`에 `AppState` 구성 및 `POST /api/update` → 새 프레임 생성 파이프라인 완성.
3. `multipart/x-mixed-replace` 스트림 엔드포인트 구현 후 로컬 테스트 (curl로 경계 파트 확인).
4. `frontend` 폼/미리보기 구현 및 백엔드 라우터에 정적파일 연결.
5. 전체 플로우 통합 테스트 및 오류/타임아웃 처리.

## 향후 고려 사항
- 다중 텍스트 노드 업데이트를 위한 배치 API (`Vec<UpdateRequest>`).
- 템플릿 핫리로드(파일 감시) 기능.
- 인증/권한 부여(필요 시 토큰 기반).
- 스트림 연결 수 제한 및 유지 관리(타임아웃, 하트비트).
