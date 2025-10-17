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
