# Feature Specification: 사용자별 세션 인증

**Feature Branch**: `002-user-session-auth`  
**Created**: 2025-10-17  
**Status**: Draft  
**Input**: User description: "사용자별 세션이 기존 공용 세션과 다른 점은 인증이 추가되어 특정 사용자만 수정이 가능하다는 점이야. /stream/{session_id} 말고 /stream/{user_id}/{session_id} 같이 유저 단위로 스트림에 접근할 수 있게 만들어야 해"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - 사용자별 세션 생성 (Priority: P1)

사용자가 자신의 계정으로 새로운 세션을 생성하고, 해당 세션의 소유권을 확보한다. 생성된 세션은 `/stream/{user_id}/{session_id}` 형태의 URL로 접근 가능하며, 다른 사용자가 해당 세션을 수정할 수 없다.

**Why this priority**: 사용자별 세션의 핵심 가치는 소유권과 접근 제어이며, 이것이 없으면 기능 자체가 성립하지 않는다.

**Independent Test**: 사용자 A가 세션을 생성하고, 생성된 세션 URL을 통해 스트림에 접근할 수 있으며, 세션 정보가 사용자 A의 소유로 저장되었는지 확인할 수 있다.

**Acceptance Scenarios**:

1. **Given** 인증된 사용자 A가 시스템에 로그인되어 있고, **When** 사용자 A가 템플릿과 파라미터를 지정하여 세션 생성 요청을 보내면, **Then** 시스템은 `{user_a_id}/{session_id}` 형태의 세션을 생성하고 소유자를 사용자 A로 기록한다
2. **Given** 사용자 A가 세션을 생성했고, **When** 사용자 A가 `/stream/{user_a_id}/{session_id}`로 접근하면, **Then** SVG 스트림이 정상적으로 표시된다
3. **Given** 사용자 A가 세션을 생성했고, **When** 사용자 B(다른 사용자)가 `/stream/{user_a_id}/{session_id}`로 접근하면, **Then** 읽기 전용으로 스트림을 볼 수 있다

---

### User Story 2 - 세션 수정 권한 검증 (Priority: P1)

세션 소유자만이 해당 세션의 파라미터를 업데이트할 수 있다. 다른 사용자가 수정을 시도하면 권한 오류가 반환된다.

**Why this priority**: 인증 기능의 핵심 보안 요구사항이며, 사용자별 세션의 존재 이유이다.

**Independent Test**: 사용자 A가 생성한 세션을 사용자 B가 수정 시도하여 거부되는지 확인하고, 사용자 A는 정상적으로 수정할 수 있는지 검증한다.

**Acceptance Scenarios**:

1. **Given** 사용자 A가 소유한 세션이 존재하고, **When** 사용자 A가 해당 세션의 파라미터를 업데이트하면, **Then** 업데이트가 성공하고 스트림에 반영된다
2. **Given** 사용자 A가 소유한 세션이 존재하고, **When** 사용자 B가 해당 세션의 파라미터를 업데이트 시도하면, **Then** 권한 오류(403 Forbidden)가 반환되고 세션은 변경되지 않는다
3. **Given** 인증되지 않은 사용자가, **When** 임의의 세션을 수정 시도하면, **Then** 인증 오류(401 Unauthorized)가 반환된다

---

### User Story 3 - 기존 공용 세션과의 호환성 유지 (Priority: P2)

기존에 사용되던 `/stream/{session_id}` 형태의 공용 세션은 계속 작동하며, 누구나 생성하고 수정할 수 있다. 사용자는 필요에 따라 공용 세션과 사용자별 세션을 선택적으로 사용할 수 있다.

**Why this priority**: 기존 사용자와 API 클라이언트의 하위 호환성을 보장하여 서비스 중단을 방지한다.

**Independent Test**: 기존 API 엔드포인트로 공용 세션을 생성하고 수정하여 정상 작동하는지 확인한다.

**Acceptance Scenarios**:

1. **Given** 시스템이 사용자별 세션 기능을 제공하고 있고, **When** 클라이언트가 기존 방식으로 `/api/session`에 세션 생성 요청을 보내면, **Then** 공용 세션이 생성되고 `/stream/{session_id}`로 접근 가능하다
2. **Given** 공용 세션이 존재하고, **When** 임의의 사용자가 해당 세션을 수정하면, **Then** 인증 없이 수정이 성공한다
3. **Given** 기존 공용 세션과 신규 사용자별 세션이 공존하고, **When** 클라이언트가 각각의 URL로 접근하면, **Then** 모두 정상적으로 작동한다

---

### User Story 4 - JWT 토큰 발급 (Priority: P0)

사용자가 시스템에서 자체적으로 JWT 토큰을 발급받을 수 있다. 시스템은 서명 키를 Redis에 안전하게 저장하고, 토큰 발급 및 검증에 사용한다.

**Why this priority**: P0 (전제 조건) - 사용자별 세션 인증의 기반이 되는 필수 인프라이며, 다른 모든 기능이 이것에 의존한다.

**Independent Test**: 사용자가 user_id를 제공하여 JWT 토큰 발급을 요청하고, 발급받은 토큰으로 인증이 필요한 API를 호출하여 정상 작동하는지 확인한다.

**Acceptance Scenarios**:

1. **Given** 시스템이 처음 시작되고 JWK(JSON Web Key)가 없을 때, **When** 시스템이 초기화되면, **Then** RSA 키 쌍을 생성하고 JWK 형태로 Redis에 저장한다
2. **Given** 사용자가 user_id를 제공하고, **When** JWT 토큰 발급을 요청하면, **Then** 시스템은 user_id를 sub claim에 포함한 JWT 토큰을 발급하고 반환한다
3. **Given** 발급된 JWT 토큰이 있고, **When** 해당 토큰으로 인증이 필요한 API를 요청하면, **Then** Redis에 저장된 JWK로 토큰을 검증하고 요청을 처리한다
4. **Given** JWT 토큰이 만료되었고, **When** 해당 토큰으로 API를 요청하면, **Then** 401 Unauthorized와 함께 토큰 만료 메시지를 반환한다

---

### User Story 5 - 사용자 세션 목록 조회 (Priority: P3)

사용자가 자신이 생성한 모든 세션의 목록을 조회할 수 있다. 이를 통해 사용자는 자신의 활성 세션을 관리하고 필요한 세션에 빠르게 접근할 수 있다.

**Why this priority**: 사용자 경험을 향상시키지만, 핵심 인증 기능 없이도 독립적으로 구현 가능한 편의 기능이다.

**Independent Test**: 사용자 A가 여러 세션을 생성한 후, 목록 조회 API를 호출하여 생성한 모든 세션이 반환되는지 확인한다.

**Acceptance Scenarios**:

1. **Given** 사용자 A가 3개의 세션을 생성했고, **When** 사용자 A가 자신의 세션 목록을 조회하면, **Then** 3개의 세션 정보(session_id, 생성 시간, 만료 시간)가 반환된다
2. **Given** 사용자 A와 사용자 B가 각각 세션을 생성했고, **When** 사용자 A가 세션 목록을 조회하면, **Then** 사용자 A의 세션만 반환되고 사용자 B의 세션은 포함되지 않는다

---

### Edge Cases

- **세션 ID 중복**: 같은 사용자가 동일한 session_id로 여러 세션을 생성하려 할 때, 기존 세션을 덮어쓸지 오류를 반환할지 결정해야 한다. (session_id는 user_id와 결합하여 고유성 보장)
- **다른 사용자 간 동일한 session_id**: 사용자 A와 사용자 B가 각각 "my-session"이라는 같은 session_id로 세션을 생성할 수 있으며, 이들은 독립적으로 관리된다
- **사용자 삭제**: 사용자 계정이 삭제되었을 때 해당 사용자의 세션들은 자동으로 만료되거나 삭제 처리된다
- **세션 만료**: TTL이 만료된 세션에 접근하려 할 때 404 Not Found 또는 410 Gone 오류가 반환된다
- **동시 수정**: 동일한 세션을 소유자가 동시에 여러 곳에서 수정할 때, 마지막 쓰기가 승리(last-write-wins) 정책을 따른다
- **비정상적인 user_id**: URL에 존재하지 않는 user_id나 잘못된 형식의 user_id가 포함될 때 404 Not Found를 반환한다
- **JWT 토큰 만료**: 유효한 세션이 존재하지만 사용자의 JWT 토큰이 만료된 경우, 세션 수정은 거부되지만 읽기는 허용된다
- **권한 에스컬레이션 시도**: URL의 user_id를 조작하여 다른 사용자의 세션에 접근하려는 시도는 JWT 토큰의 사용자 정보와 비교하여 차단된다
- **JWK 키 손실**: Redis에서 JWK가 삭제되거나 손실된 경우, 새로운 키 쌍을 생성하고 기존 토큰은 모두 무효화된다
- **동시 키 생성**: 여러 서버 인스턴스가 동시에 시작될 때 JWK 생성 경쟁 조건이 발생하지 않도록 원자적 연산을 사용한다
- **토큰 재사용**: 동일한 user_id로 여러 번 토큰을 발급받을 수 있으며, 각 토큰은 독립적으로 유효하다

## Requirements *(mandatory)*

### Functional Requirements

#### JWT 토큰 발급 및 관리
- **FR-001**: 시스템은 시작 시 Redis에 JWK(JSON Web Key)가 없으면 RSA-2048 키 쌍을 생성하고 jsonwebtoken 라이브러리의 Jwk 객체로 Redis에 저장해야 한다
- **FR-002**: 시스템은 jsonwebtoken::Jwk 객체를 serde_json으로 직렬화하여 Redis `jwk:private` 및 `jwk:public` 키로 저장해야 한다
- **FR-003**: 시스템은 Redis에서 로드한 Jwk 객체를 메모리에 캐싱하여 매 요청마다 Redis 조회를 방지해야 한다
- **FR-004**: 시스템은 JWK 메모리 캐시를 애플리케이션 시작 시 초기화하고, Redis에서 키를 로드해야 한다
- **FR-005**: 시스템은 JWT 토큰 발급 API(`POST /api/auth/token`)를 제공해야 한다
- **FR-006**: 시스템은 토큰 발급 시 요청 본문의 user_id를 JWT의 sub claim에 포함해야 한다
- **FR-007**: 시스템은 발급하는 JWT 토큰에 만료 시간(exp), 발급 시간(iat), 발급자(iss) claim을 포함해야 한다
- **FR-008**: 시스템은 JWK 키 쌍 생성 시 원자적 연산(SET NX)을 사용하여 동시 생성을 방지해야 한다
- **FR-009**: 시스템은 JWKS 표준 엔드포인트(`GET /.well-known/jwks.json`)를 제공하여 메모리 캐시된 JWK 공개 키 정보를 반환해야 한다
- **FR-010**: 시스템은 JWT 토큰 검증 시 메모리 캐시된 Jwk 객체를 사용해야 한다

#### 사용자별 세션 관리
- **FR-011**: 시스템은 사용자별 세션 생성 API(`POST /api/user/{user_id}/session`)를 제공해야 한다
- **FR-012**: 시스템은 사용자별 스트림 엔드포인트(`GET /stream/{user_id}/{session_id}`)를 제공해야 한다
- **FR-013**: 시스템은 세션 생성 시 소유자(user_id)를 세션 메타데이터에 저장해야 한다
- **FR-014**: 시스템은 세션 수정 요청(`PUT /api/user/{user_id}/session/{session_id}`) 시 JWT 토큰에서 추출한 사용자 정보와 URL의 user_id가 일치하는지 검증해야 한다
- **FR-015**: 시스템은 JWT 토큰의 사용자와 세션 소유자가 일치하지 않는 경우 403 Forbidden을 반환해야 한다
- **FR-016**: 시스템은 유효하지 않거나 만료된 JWT 토큰으로 세션 수정 시도 시 401 Unauthorized를 반환해야 한다
- **FR-017**: 시스템은 사용자별 세션을 Redis에 `user:{user_id}:session:{session_id}` 형태의 키로 저장해야 한다
- **FR-018**: 시스템은 사용자가 생성한 세션 목록 조회 API(`GET /api/user/{user_id}/sessions`)를 제공해야 한다
- **FR-019**: 시스템은 사용자별 세션의 pubsub 채널을 `user:{user_id}:session:{session_id}` 형태로 사용해야 한다

#### 공용 세션 및 호환성
- **FR-020**: 시스템은 기존 공용 세션 API(`POST /api/session`, `GET /stream/{session_id}`)를 계속 지원해야 한다
- **FR-021**: 시스템은 공용 세션에 대해서는 JWT 토큰 없이도 수정을 허용해야 한다
- **FR-022**: 시스템은 공용 세션을 Redis에 `session:{session_id}` 형태의 키로 저장해야 한다

#### 인증 및 권한 검증
- **FR-023**: 시스템은 세션 읽기(스트림 조회)는 JWT 토큰 없이도 허용해야 한다 (공개 읽기)
- **FR-024**: 시스템은 Authorization 헤더의 Bearer 토큰에서 JWT를 추출하고 메모리 캐시된 JWK로 검증해야 한다
- **FR-025**: 시스템은 JWT 토큰에서 사용자 식별자(sub claim)를 추출할 수 있어야 한다
- **FR-026**: 시스템은 토큰 검증 실패 시 구체적인 오류 원인(만료, 서명 불일치, 형식 오류)을 로그에 기록해야 한다

### Key Entities

- **JWK (JSON Web Key)**: 시스템의 서명 키 쌍. RSA-2048 비트 공개/비밀 키로 구성되며, jsonwebtoken::Jwk 객체로 Redis에 저장된다. 비밀 키는 JWT 서명에, 공개 키는 검증에 사용된다. Redis 키: `jwk:private`, `jwk:public`
- **JWT Token**: 사용자 인증 정보를 포함하는 토큰. sub(user_id), exp(만료 시간), iat(발급 시간), iss(발급자) claim을 포함하며, JWK의 비밀 키로 서명된다
- **User Session**: 특정 사용자가 소유한 세션. user_id(소유자), session_id, template, args, 생성 시간, TTL을 포함한다. Redis 키: `user:{user_id}:session:{session_id}`
- **Public Session**: 소유자가 없는 공용 세션. session_id, template, args, 생성 시간, TTL을 포함한다. Redis 키: `session:{session_id}`
- **User**: 시스템 사용자. JWT 토큰의 sub claim으로 식별되며, 여러 개의 User Session을 소유할 수 있다

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 사용자가 JWT 토큰을 발급받는 데 1초 이내가 소요된다
- **SC-002**: 시스템 시작 시 JWK 생성, Redis 저장 및 메모리 캐싱이 5초 이내에 완료된다
- **SC-003**: JWK 메모리 캐시 사용으로 JWT 토큰 검증 시 Redis 조회가 0회 발생한다
- **SC-004**: 사용자가 토큰을 발급받고 자신의 세션을 생성하여 3초 이내에 스트림에 접근할 수 있다
- **SC-005**: 권한이 없는 사용자의 세션 수정 시도는 100% 차단된다
- **SC-006**: 기존 공용 세션 API 사용자들은 코드 변경 없이 계속 서비스를 이용할 수 있다
- **SC-007**: 사용자당 최소 100개의 세션을 동시에 관리할 수 있다
- **SC-008**: JWT 토큰 검증 및 권한 확인 시간이 50ms 이하이다 (메모리 캐시 사용)
- **SC-009**: 세션 목록 조회 시 100개의 세션을 1초 이내에 반환할 수 있다

## Assumptions

- 시스템은 자체적으로 JWT 토큰을 발급하며, 외부 인증 제공자를 사용하지 않음
- JWK는 시스템 시작 시 자동으로 생성되고 JWK 형식으로 Redis에 영구 저장됨 (TTL 없음)
- JWK는 Redis에서 로드 후 애플리케이션 메모리에 캐싱되며, 재시작 전까지 유지됨
- 메모리 캐시된 JWK는 애플리케이션 인스턴스 전체에서 공유되는 단일 인스턴스임
- Redis가 재시작되어 JWK가 손실되면 새로운 키 쌍을 생성하고, 기존 토큰은 모두 무효화됨
- JWT 토큰 검증 시 매번 Redis 조회 대신 메모리 캐시된 JWK를 사용하여 성능 최적화
- JWT 토큰의 기본 만료 시간은 24시간(86400초)임
- 사용자 ID는 JWT 토큰의 `sub` (subject) claim에 포함됨
- Redis 저장소는 패턴 매칭(`user:{user_id}:session:*`)을 통한 키 검색을 지원함
- 세션 TTL은 사용자별 세션과 공용 세션 모두 동일한 정책을 따름 (기본 3600초)
- 동일한 user_id와 session_id 조합으로 세션을 재생성하면 기존 세션을 덮어쓴다 (Redis SET 동작)
- 사용자 간에는 동일한 session_id 사용이 허용됨 (user_id로 네임스페이스 분리)
- 토큰 발급 시 사용자 존재 여부나 비밀번호 검증을 수행하지 않음 (단순 토큰 발급만 수행)
