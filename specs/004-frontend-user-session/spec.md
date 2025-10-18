# Feature Specification: 프론트엔드 유저 세션 UI 통합

**Feature Branch**: `004-frontend-user-session`  
**Created**: 2025-10-18  
**Status**: Draft  
**Input**: User description: "프론트엔드에서 백엔드의 유저별 세션 기능 사용"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - JWT 토큰 발급 UI (Priority: P0)

프론트엔드에서 사용자가 user_id를 입력하여 JWT 토큰을 발급받을 수 있다. 발급받은 토큰은 브라우저에 안전하게 저장되고, 이후 인증이 필요한 모든 API 요청에 자동으로 포함된다.

**Why this priority**: P0 (전제 조건) - 유저별 세션 생성 및 수정의 필수 전제 조건이며, 이 기능 없이는 다른 모든 기능을 사용할 수 없다.

**Independent Test**: 사용자가 user_id를 입력하고 토큰을 발급받은 후, 브라우저 저장소에 토큰이 저장되었는지 확인하고, 이후 API 요청 시 Authorization 헤더에 토큰이 포함되는지 검증한다.

**Acceptance Scenarios**:

1. **Given** 프론트엔드 앱이 로드되고 토큰이 없는 상태일 때, **When** 사용자가 user_id를 입력하고 "토큰 발급" 버튼을 클릭하면, **Then** 백엔드로 `POST /api/auth/token` 요청이 전송되고 JWT 토큰이 발급된다
2. **Given** 토큰이 성공적으로 발급되었을 때, **When** 토큰을 수신하면, **Then** 브라우저 localStorage에 토큰이 저장되고 UI에 로그인 상태가 표시된다
3. **Given** 사용자가 로그인한 상태일 때, **When** 세션 생성 또는 수정 API를 호출하면, **Then** 요청 헤더에 `Authorization: Bearer {token}` 형태로 토큰이 자동 포함된다
4. **Given** localStorage에 저장된 토큰이 있을 때, **When** 페이지를 새로고침하면, **Then** 토큰이 자동으로 로드되어 로그인 상태가 유지된다

---

### User Story 2 - 유저별 세션 생성 UI (Priority: P1)

사용자가 자신의 계정으로 세션을 생성할 수 있도록 UI가 구성된다. 세션 생성 양식에서 사용자는 session_id, template, args, expire를 입력하고, JWT 토큰을 사용해 `/api/user/{user_id}/session` API를 호출하여 자신만의 세션을 생성한다.

**Why this priority**: P1 - 유저별 세션의 핵심 가치를 제공하는 첫 번째 실질적 기능이다.

**Independent Test**: 로그인한 사용자가 세션 생성 페이지에서 정보를 입력하고 "세션 생성" 버튼을 클릭하여, 백엔드에 세션이 생성되고 `/stream/{user_id}/{session_id}` URL로 접근 가능한지 확인한다.

**Acceptance Scenarios**:

1. **Given** 사용자 A가 로그인된 상태이고, **When** 세션 생성 페이지에서 session_id, template을 입력하고 "세션 생성" 버튼을 클릭하면, **Then** `POST /api/user/alice/session` 요청이 JWT 토큰과 함께 전송되고 세션이 생성된다
2. **Given** 세션 생성이 성공했을 때, **When** 응답을 수신하면, **Then** UI는 세션 상세 페이지로 자동 리다이렉트되고 `/session/{user_id}/{session_id}` URL로 이동한다
3. **Given** 사용자가 로그인하지 않은 상태이고, **When** 유저별 세션 생성을 시도하면, **Then** "로그인이 필요합니다" 메시지가 표시되고 토큰 발급 UI가 안내된다
4. **Given** 세션 생성 중 오류가 발생했을 때, **When** 백엔드가 401/403/409 오류를 반환하면, **Then** UI는 사용자에게 적절한 오류 메시지를 표시한다

---

### User Story 3 - 유저별 세션 수정 UI (Priority: P1)

사용자가 자신이 생성한 세션의 상세 페이지에서 파라미터를 수정할 수 있다. 수정 시 JWT 토큰이 자동으로 포함되어 `/api/user/{user_id}/session/{session_id}` API를 호출하며, 소유자가 아닌 경우 수정이 차단된다.

**Why this priority**: P1 - 세션 생성 후 실시간으로 콘텐츠를 업데이트하는 핵심 사용 사례이다.

**Independent Test**: 로그인한 사용자 A가 자신의 세션 상세 페이지에서 파라미터를 수정하고 "업데이트" 버튼을 클릭하여 성공적으로 업데이트되는지, 다른 사용자 B는 수정이 거부되는지 확인한다.

**Acceptance Scenarios**:

1. **Given** 사용자 A가 로그인하고 자신의 세션 상세 페이지에 있을 때, **When** 파라미터 JSON을 수정하고 "업데이트" 버튼을 클릭하면, **Then** `PUT /api/user/alice/session-1` 요청이 JWT 토큰과 함께 전송되고 파라미터가 업데이트된다
2. **Given** 업데이트가 성공했을 때, **When** 응답을 수신하면, **Then** UI는 "업데이트가 완료되었습니다" 메시지를 표시하고 스트림 미리보기가 새로고침된다
3. **Given** 사용자 B가 로그인하고 사용자 A의 세션 상세 페이지에 접근했을 때, **When** 수정을 시도하면, **Then** 백엔드가 403 오류를 반환하고 UI는 "권한이 없습니다" 메시지를 표시한다
4. **Given** JWT 토큰이 만료되었을 때, **When** 세션 수정을 시도하면, **Then** 백엔드가 401 오류를 반환하고 UI는 "토큰이 만료되었습니다. 다시 로그인하세요" 메시지를 표시한다

---

### User Story 4 - 유저별 세션 목록 조회 UI (Priority: P2)

사용자가 자신이 생성한 모든 세션 목록을 한눈에 조회하고, 원하는 세션을 선택하여 상세 페이지로 이동할 수 있다.

**Why this priority**: P2 - 사용자 경험을 향상시키지만, 세션 생성/수정 기능이 먼저 구현되어야 실질적인 가치를 제공할 수 있다.

**Independent Test**: 로그인한 사용자가 "내 세션 목록" 페이지에 접근하여, 자신이 생성한 세션들이 목록으로 표시되고 각 세션을 클릭하면 상세 페이지로 이동하는지 확인한다.

**Acceptance Scenarios**:

1. **Given** 사용자 A가 로그인하고 3개의 세션을 생성했을 때, **When** "내 세션 목록" 페이지로 이동하면, **Then** `GET /api/user/alice/sessions` 요청이 JWT 토큰과 함께 전송되고 3개의 세션이 목록으로 표시된다
2. **Given** 세션 목록이 표시되었을 때, **When** 특정 세션을 클릭하면, **Then** 해당 세션의 상세 페이지(`/session/alice/session-1`)로 이동한다
3. **Given** 사용자가 세션을 생성하지 않았을 때, **When** 세션 목록 페이지에 접근하면, **Then** "생성된 세션이 없습니다" 메시지와 함께 "새 세션 만들기" 버튼이 표시된다
4. **Given** 세션 목록에 각 세션의 메타데이터가 표시될 때, **When** 목록을 보면, **Then** 각 세션의 session_id, 생성 시간, 남은 TTL이 표시된다

---

### User Story 5 - 공용 세션 호환성 유지 (Priority: P1)

기존 공용 세션(`POST /api/session`, `/stream/{session_id}`) 기능을 계속 사용할 수 있다. 사용자는 로그인 여부와 관계없이 공용 세션을 생성하고 수정할 수 있으며, 유저별 세션과 공용 세션을 선택적으로 사용할 수 있다.

**Why this priority**: P1 - 기존 사용자의 워크플로우를 유지하고 하위 호환성을 보장하여 점진적인 마이그레이션을 가능하게 한다.

**Independent Test**: 로그인하지 않은 사용자가 기존 방식으로 세션을 생성하고 수정하여 정상 작동하는지 확인한다.

**Acceptance Scenarios**:

1. **Given** 사용자가 로그인하지 않은 상태이고, **When** 기존 세션 생성 페이지에서 session_id와 template을 입력하고 "세션 생성"을 클릭하면, **Then** `POST /api/session` 요청이 전송되고 공용 세션이 생성된다
2. **Given** 공용 세션이 생성되었을 때, **When** 상세 페이지로 이동하면, **Then** `/session/{session_id}` URL에서 세션을 조회하고 수정할 수 있다
3. **Given** 공용 세션 상세 페이지에 있을 때, **When** 파라미터를 수정하고 "업데이트"를 클릭하면, **Then** JWT 토큰 없이 `PUT /api/session/{session_id}` 요청이 성공한다
4. **Given** UI에 유저별 세션과 공용 세션 모드가 구분되어 있을 때, **When** 사용자가 모드를 전환하면, **Then** 각 모드에 맞는 API 엔드포인트와 URL 형식이 사용된다

---

### User Story 6 - 로그아웃 및 토큰 관리 (Priority: P2)

사용자가 로그아웃하여 저장된 JWT 토큰을 삭제하고 비로그인 상태로 전환할 수 있다. 로그아웃 후에도 공용 세션은 계속 사용할 수 있다.

**Why this priority**: P2 - 보안과 사용자 경험을 위해 중요하지만, 핵심 세션 관리 기능이 먼저 구현되어야 한다.

**Independent Test**: 로그인한 사용자가 "로그아웃" 버튼을 클릭하여 localStorage에서 토큰이 삭제되고, 이후 유저별 세션 생성 시도 시 로그인이 요구되는지 확인한다.

**Acceptance Scenarios**:

1. **Given** 사용자가 로그인된 상태이고, **When** "로그아웃" 버튼을 클릭하면, **Then** localStorage에서 JWT 토큰이 삭제되고 UI가 비로그인 상태로 전환된다
2. **Given** 로그아웃 후, **When** 유저별 세션 생성을 시도하면, **Then** "로그인이 필요합니다" 메시지가 표시되고 토큰 발급 UI가 안내된다
3. **Given** 로그아웃 후, **When** 공용 세션 생성을 시도하면, **Then** 토큰 없이 정상적으로 공용 세션이 생성된다
4. **Given** 로그인 상태 표시가 UI에 있을 때, **When** 로그아웃하면, **Then** 현재 user_id 표시가 사라지고 "로그인" 버튼이 표시된다

---

### Edge Cases

- **토큰 만료 중 작업**: 사용자가 세션 수정 중에 JWT 토큰이 만료되면 401 오류가 발생하고, UI는 "토큰이 만료되었습니다. 다시 로그인하세요" 메시지를 표시한다
- **잘못된 user_id 입력**: 토큰 발급 시 user_id가 비어있거나 공백만 있을 경우 "유효한 user_id를 입력하세요" 오류가 표시된다
- **네트워크 오류**: API 요청 중 네트워크 오류 발생 시 "네트워크 오류가 발생했습니다. 다시 시도하세요" 메시지를 표시한다
- **URL 직접 접근**: 로그인하지 않은 사용자가 `/session/{user_id}/{session_id}` URL로 직접 접근할 경우, 스트림은 볼 수 있지만 수정은 차단된다 (읽기 전용)
- **다른 사용자 세션 접근**: 사용자 A가 사용자 B의 세션 상세 페이지에 접근하면, 세션 정보는 표시되지만 "업데이트" 버튼이 비활성화되거나 수정 시도 시 403 오류가 표시된다
- **세션 ID 충돌**: 사용자가 이미 존재하는 session_id로 세션 생성 시도 시 409 오류와 함께 "이미 존재하는 세션 ID입니다" 메시지가 표시된다
- **localStorage 사용 불가**: 브라우저가 localStorage를 지원하지 않거나 비활성화된 경우, "브라우저 설정을 확인하세요" 메시지를 표시한다
- **JSON 파싱 오류**: 사용자가 args 입력란에 잘못된 JSON을 입력하면 "JSON 구문 오류: [오류 내용]" 메시지가 표시된다
- **공용 세션과 유저 세션 URL 혼동**: `/session/{session_id}` 형태의 URL로 공용 세션에 접근하고, `/session/{user_id}/{session_id}` 형태로 유저 세션에 접근하며, 잘못된 형식은 404를 반환한다

## Requirements *(mandatory)*

### Functional Requirements

#### JWT 토큰 발급 및 저장
- **FR-001**: UI는 사용자가 user_id를 입력할 수 있는 텍스트 입력란과 "토큰 발급" 버튼을 제공해야 한다
- **FR-002**: "토큰 발급" 버튼 클릭 시 `POST /api/auth/token` API를 호출하고 응답에서 JWT 토큰을 추출해야 한다
- **FR-003**: 발급받은 JWT 토큰은 브라우저 localStorage에 `jwt_token` 키로 저장되어야 한다
- **FR-004**: 페이지 로드 시 localStorage에서 토큰을 자동으로 읽어와 로그인 상태를 복원해야 한다
- **FR-005**: localStorage에 저장된 토큰이 있으면 UI에 로그인 상태를 표시하고, 현재 user_id를 추출하여 표시해야 한다 (JWT의 sub claim 디코딩)
- **FR-006**: 토큰 발급 중 오류 발생 시 사용자에게 오류 메시지를 표시해야 한다

#### 인증 헤더 자동 추가
- **FR-007**: 유저별 세션 API 호출 시 (`POST /api/user/{user_id}/session`, `PUT /api/user/{user_id}/session/{session_id}`, `GET /api/user/{user_id}/sessions`) localStorage의 토큰을 읽어 `Authorization: Bearer {token}` 헤더를 자동으로 추가해야 한다
- **FR-008**: 공용 세션 API 호출 시 (`POST /api/session`, `PUT /api/session/{session_id}`) Authorization 헤더를 포함하지 않아야 한다
- **FR-009**: API 요청 시 401 Unauthorized 응답을 받으면 "토큰이 만료되었습니다" 메시지를 표시하고 로그아웃 처리를 수행해야 한다

#### 유저별 세션 생성 UI
- **FR-010**: 유저별 세션 생성 양식은 session_id, template, args (선택), expire (선택) 입력란을 제공해야 한다
- **FR-011**: 유저별 세션 생성 양식 제출 시 로그인 상태를 확인하고, 비로그인 상태이면 "로그인이 필요합니다" 메시지를 표시해야 한다
- **FR-012**: 로그인 상태에서 양식 제출 시 localStorage에서 JWT 토큰을 읽어 user_id(sub claim)를 추출하고 `POST /api/user/{user_id}/session` API를 호출해야 한다
- **FR-013**: 유저별 세션 생성 성공 시 `/session/{user_id}/{session_id}` URL로 리다이렉트되어야 한다
- **FR-014**: 유저별 세션 생성 중 409 오류 발생 시 "이미 존재하는 세션 ID입니다" 메시지를 표시해야 한다

#### 유저별 세션 상세 페이지 및 수정
- **FR-015**: 유저별 세션 상세 페이지는 `/session/{user_id}/{session_id}` URL 패턴으로 라우팅되어야 한다
- **FR-016**: 유저별 세션 상세 페이지 로드 시 `GET /api/user/{user_id}/session/{session_id}` API를 호출하여 세션 정보를 조회해야 한다 (읽기는 인증 불필요)
- **FR-017**: 유저별 세션 스트림 미리보기는 `/stream/{user_id}/{session_id}` URL을 사용해야 한다
- **FR-018**: 유저별 세션 파라미터 수정 시 로그인 상태를 확인하고, JWT 토큰을 포함하여 `PUT /api/user/{user_id}/session/{session_id}` API를 호출해야 한다
- **FR-019**: 유저별 세션 수정 시 403 오류 발생 시 "권한이 없습니다. 세션 소유자만 수정할 수 있습니다" 메시지를 표시해야 한다
- **FR-020**: 유저별 세션 수정 시 401 오류 발생 시 "토큰이 만료되었습니다. 다시 로그인하세요" 메시지를 표시하고 로그아웃 처리를 수행해야 한다

#### 유저별 세션 목록 조회
- **FR-021**: "내 세션 목록" 페이지를 제공하고, `/my-sessions` URL로 라우팅되어야 한다
- **FR-022**: 로그인 상태에서 세션 목록 페이지 로드 시 localStorage에서 user_id를 추출하고 `GET /api/user/{user_id}/sessions` API를 JWT 토큰과 함께 호출해야 한다
- **FR-023**: 세션 목록은 각 세션의 session_id, 생성 시간, 남은 TTL을 표시해야 한다
- **FR-024**: 세션 목록에서 세션을 클릭하면 해당 유저별 세션 상세 페이지(`/session/{user_id}/{session_id}`)로 이동해야 한다
- **FR-025**: 세션 목록이 비어있을 때 "생성된 세션이 없습니다" 메시지와 "새 세션 만들기" 버튼을 표시해야 한다

#### 공용 세션 호환성
- **FR-026**: 공용 세션 생성 양식은 기존과 동일하게 session_id, template, args, expire 입력란을 제공해야 한다
- **FR-027**: 공용 세션 생성 시 `POST /api/session` API를 JWT 토큰 없이 호출해야 한다
- **FR-028**: 공용 세션 생성 성공 시 `/session/{session_id}` URL로 리다이렉트되어야 한다
- **FR-029**: 공용 세션 상세 페이지는 `/session/{session_id}` URL 패턴으로 라우팅되어야 한다 (기존과 동일)
- **FR-030**: 공용 세션 수정 시 `PUT /api/session/{session_id}` API를 JWT 토큰 없이 호출해야 한다
- **FR-031**: 공용 세션 스트림 미리보기는 `/stream/{session_id}` URL을 사용해야 한다 (기존과 동일)

#### 로그아웃 및 상태 관리
- **FR-032**: UI 헤더에 "로그아웃" 버튼을 제공하고, 로그인 상태일 때만 표시되어야 한다
- **FR-033**: "로그아웃" 버튼 클릭 시 localStorage에서 `jwt_token` 키를 삭제하고 UI를 비로그인 상태로 전환해야 한다
- **FR-034**: 로그아웃 후 홈 페이지(`/`)로 리다이렉트되어야 한다
- **FR-035**: UI는 로그인 상태와 비로그인 상태를 명확히 구분하여 표시해야 한다 (예: 헤더에 현재 user_id 표시)

#### URL 라우팅 및 네비게이션
- **FR-036**: URL 라우팅은 다음 패턴을 지원해야 한다:
  - `/`: 홈 페이지 (세션 생성)
  - `/session/{session_id}`: 공용 세션 상세 페이지
  - `/session/{user_id}/{session_id}`: 유저별 세션 상세 페이지
  - `/my-sessions`: 내 세션 목록 페이지
- **FR-037**: URL 패턴 분석을 통해 공용 세션과 유저별 세션을 자동으로 구분해야 한다 (세그먼트 개수로 판단)
- **FR-038**: 잘못된 URL 패턴 접근 시 404 에러 페이지를 표시하거나 홈으로 리다이렉트해야 한다

#### 에러 처리 및 사용자 피드백
- **FR-039**: 모든 API 오류는 사용자에게 이해하기 쉬운 메시지로 변환하여 표시해야 한다
- **FR-040**: 네트워크 오류 발생 시 "네트워크 오류가 발생했습니다. 다시 시도하세요" 메시지를 표시해야 한다
- **FR-041**: JSON 파싱 오류 발생 시 "JSON 구문 오류: [오류 내용]" 메시지를 표시해야 한다
- **FR-042**: localStorage 사용 불가 시 "브라우저 설정을 확인하세요" 메시지를 표시해야 한다

### Key Entities

- **JWT Token**: 사용자 인증 정보를 포함하는 토큰. localStorage에 `jwt_token` 키로 저장되며, sub claim에 user_id를 포함한다. 만료 시간(exp)이 있으며, Authorization 헤더로 전송된다.
- **User Session**: 특정 사용자가 소유한 세션. user_id, session_id, template, args, 생성 시간, TTL을 포함한다. `/stream/{user_id}/{session_id}` URL로 접근 가능하다.
- **Public Session**: 소유자가 없는 공용 세션. session_id, template, args, 생성 시간, TTL을 포함한다. `/stream/{session_id}` URL로 접근 가능하다.
- **Login State**: 프론트엔드의 로그인 상태. localStorage의 토큰 존재 여부로 판단하며, 토큰에서 user_id를 추출하여 UI에 표시한다.
- **Session List Item**: 세션 목록의 항목. session_id, 생성 시간(created_at), 남은 TTL을 포함한다.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 사용자가 user_id를 입력하고 토큰을 발급받는 데 3초 이내가 소요된다
- **SC-002**: 페이지 로드 시 localStorage에서 토큰을 읽어 로그인 상태를 복원하는 데 1초 이내가 소요된다
- **SC-003**: 로그인한 사용자가 유저별 세션을 생성하고 상세 페이지로 이동하는 데 5초 이내가 소요된다
- **SC-004**: 유저별 세션 파라미터 업데이트 후 스트림에 변경 사항이 반영되는 데 2초 이내가 소요된다
- **SC-005**: 세션 목록 페이지에서 100개의 세션을 표시하는 데 3초 이내가 소요된다
- **SC-006**: 기존 공용 세션 사용자는 코드 변경 없이 계속 서비스를 이용할 수 있다
- **SC-007**: 유저별 세션 생성/수정 시도 시 권한 오류(401/403)가 100% 정확하게 감지되고 사용자에게 알림된다
- **SC-008**: 모든 API 오류는 사용자에게 이해 가능한 메시지로 표시되어 사용자 만족도가 향상된다
- **SC-009**: 토큰 만료 시 사용자는 명확한 안내 메시지를 받고 재로그인할 수 있다

## Assumptions

- 프론트엔드는 Yew 프레임워크와 WASM을 사용하여 구현되며, 기존 코드 스타일을 유지한다
- JWT 토큰은 브라우저 localStorage에 평문으로 저장된다 (HTTPS 사용 가정)
- JWT 토큰의 sub claim에 user_id가 포함되어 있어 클라이언트에서 디코딩하여 추출할 수 있다
- 백엔드 API(`/api/auth/token`, `/api/user/{user_id}/session`, etc.)는 이미 구현되어 있으며, spec 002의 계약을 따른다
- 공용 세션과 유저별 세션은 URL 패턴으로 구분 가능하다 (세그먼트 개수: 1 vs 2)
- 세션 읽기(스트림 조회)는 JWT 토큰 없이도 가능하다 (공개 읽기)
- 세션 수정은 소유자만 가능하며, JWT 토큰으로 소유권을 검증한다
- localStorage가 비활성화된 브라우저 환경에서는 기능이 제한될 수 있으나, 공용 세션은 계속 사용 가능하다
- 토큰 만료 시 자동 갱신 기능은 제공하지 않으며, 사용자가 수동으로 재로그인해야 한다
- URL 라우팅은 브라우저의 History API를 사용하여 SPA 방식으로 구현된다
- 세션 목록 API는 페이지네이션을 제공하지 않으며, 모든 세션을 한 번에 반환한다 (성능 고려는 향후 개선)
- JWT 토큰 디코딩은 클라이언트 사이드에서 base64 디코딩으로 수행하며, 서명 검증은 백엔드에서만 수행한다
