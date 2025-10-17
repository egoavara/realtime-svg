# realtime-svg 설계 메모 (multipart/x-mixed-replace)

연제나 한국어로 대답한다

## 프로젝트 목표
- `multipart/x-mixed-replace` 스트리밍 응답을 활용해 SVG 이미지를 실시간으로 갱신한다.
- SVG 내 특정 텍스트 노드 값이 서버 측 변경 이벤트마다 자동으로 덮어씌워지도록 한다.
- Rust 기반 멀티 크레이트(workspace) 구조를 유지하면서, 공통 로직과 프런트엔드/백엔드 역할을 명확히 분리한다.
