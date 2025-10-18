# realtime-svg

실시간 SVG 스트리밍 서버 with JWT 인증

## 개요

`multipart/x-mixed-replace` 프로토콜을 사용하여 SVG 이미지를 실시간으로 브라우저에 스트리밍하는 Rust 기반 서버입니다. Tera 템플릿 엔진으로 동적 SVG를 생성하고, Redis를 통해 실시간 파라미터 업데이트를 브로드캐스트합니다.

**주요 기능:**
- 🔐 JWT 기반 사용자 인증 (RSA-2048)
- 👤 사용자별 세션 관리 및 소유권 제어
- 📡 실시간 SVG 스트리밍 (`multipart/x-mixed-replace`)
- 🎨 Tera 템플릿 기반 동적 SVG 생성
- 🚀 Redis pubsub을 통한 실시간 브로드캐스트
- 🔄 기존 공용 세션 API 완전 호환

## 아키텍처

```
┌─────────────┐          ┌──────────────┐          ┌─────────┐
│   Browser   │◄─────────│   Backend    │◄─────────│  Redis  │
│             │  Stream  │   (Axum)     │  pubsub  │         │
└──────┬──────┘          └──────┬───────┘          └────┬────┘
       │                        │                       │
       │ 1. POST /api/auth/token                      │
       │─────────────────────►  │                       │
       │◄─────────────────────  │  (JWT issued)        │
       │ {"token": "..."}       │                       │
       │                        │                       │
       │ 2. POST /api/user/{user_id}/session          │
       │    Authorization: Bearer <token>              │
       │─────────────────────►  │                       │
       │                        │  SET user:alice:...   │
       │                        │──────────────────────►│
       │◄─────────────────────  │                       │
       │ {"session_id": "..."}  │                       │
       │                        │                       │
       │ 3. GET /stream/{user_id}/{session_id}        │
       │─────────────────────►  │                       │
       │                        │  SUBSCRIBE ...        │
       │                        │──────────────────────►│
       │◄═════════════════════  │                       │
       │   SVG Stream (live)    │◄══════════════════════│
       │                        │  (pubsub updates)     │
       │                        │                       │
       │ 4. PUT /api/user/{user_id}/session/{id}      │
       │    Authorization: Bearer <token>              │
       │─────────────────────►  │                       │
       │                        │  PUBLISH update       │
       │                        │──────────────────────►│
       │                        │                       │
       │   (All connected clients get updated SVG)     │
       │◄═════════════════════  │◄══════════════════════│
└────────────────────────────────────────────────────────┘
```

## JWT 인증 플로우

```
┌──────────┐                 ┌──────────┐                 ┌─────────┐
│  Client  │                 │  Server  │                 │  Redis  │
└────┬─────┘                 └────┬─────┘                 └────┬────┘
     │                            │                            │
     │ POST /api/auth/token       │                            │
     │ {"user_id": "alice"}       │                            │
     │───────────────────────────►│                            │
     │                            │ GET rsa:private_pem        │
     │                            │───────────────────────────►│
     │                            │◄───────────────────────────│
     │                            │ (JwkCache: first load)     │
     │                            │                            │
     │                            │ Sign JWT with RSA key      │
     │                            │ {sub: "alice",             │
     │                            │  exp: now + 24h,           │
     │                            │  iss: "realtime-svg"}      │
     │◄───────────────────────────│                            │
     │ {"token": "eyJhbG..."}     │                            │
     │                            │                            │
     │ POST /api/user/alice/session                           │
     │ Authorization: Bearer eyJ...                            │
     │───────────────────────────►│                            │
     │                            │ GET rsa:public_pem         │
     │                            │ (cached in JwkCache)       │
     │                            │                            │
     │                            │ Verify signature ✓         │
     │                            │ Check exp > now ✓          │
     │                            │ Check iss == "realtime-svg"✓
     │                            │                            │
     │                            │ Extract sub = "alice"      │
     │                            │ Match URL user_id ✓        │
     │                            │                            │
     │                            │ SET user:alice:session:... │
     │                            │───────────────────────────►│
     │◄───────────────────────────│                            │
     │ 201 Created                │                            │
     │                            │                            │
```

## 빠른 시작

### 사전 요구사항

- Rust 1.75+
- Redis 서버

### 설치 및 실행

```bash
# 1. Redis 실행
redis-server

# 2. 프로젝트 빌드
cargo build --release

# 3. 서버 실행
cargo run --bin backend

# Server listening on http://0.0.0.0:3000
```

### 사용 예제

#### 1. JWT 토큰 발급

```bash
curl -X POST http://localhost:3000/api/auth/token \
  -H "Content-Type: application/json" \
  -d '{"user_id": "alice", "password": "your-password"}'

# Response: {"token": "eyJhbGciOiJSUzI1NiIs..."}
```

#### 2. 사용자 세션 생성

```bash
export TOKEN="eyJhbGciOiJSUzI1NiIs..."

curl -X POST http://localhost:3000/api/user/alice/session \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "session_id": "dashboard-1",
    "template": "<svg><text fill=\"{{color}}\">{{status}}</text></svg>",
    "args": {"status": "online", "color": "green"}
  }'
```

#### 3. 브라우저에서 스트림 보기

```
http://localhost:3000/stream/alice/dashboard-1
```

#### 4. 실시간 업데이트

```bash
curl -X PUT http://localhost:3000/api/user/alice/session/dashboard-1 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"args": {"status": "offline", "color": "red"}}'
```

→ 모든 연결된 브라우저가 즉시 빨간색 "offline" 메시지를 표시합니다.

## API 문서

### 인증 API

#### `POST /api/auth/token`
JWT 토큰 발급

**Request:**
```json
{
  "user_id": "alice",
  "password": "your-password",
  "ttl_seconds": 86400  // optional, default: 3600 seconds
}
```

**Response:**
```json
{
  "token": "eyJhbGciOiJSUzI1NiIs..."
}
```

#### `GET /.well-known/jwks.json`
JWK 공개 키 조회 (RFC 8414)

**Response:**
```json
{
  "keys": [
    {
      "kty": "RSA",
      "n": "xGOr...",
      "e": "AQAB",
      "alg": "RS256",
      "use": "sig"
    }
  ]
}
```

### 사용자 세션 API

#### `POST /api/user/{user_id}/session`
사용자 세션 생성 (JWT 필요)

**Headers:**
- `Authorization: Bearer <token>`

**Request:**
```json
{
  "session_id": "dashboard-1",
  "template": "<svg>...</svg>",
  "args": {"key": "value"}
}
```

#### `PUT /api/user/{user_id}/session/{session_id}`
세션 파라미터 업데이트 (소유자만 가능)

**Headers:**
- `Authorization: Bearer <token>`

**Request:**
```json
{
  "args": {"status": "updated"}
}
```

#### `GET /api/user/{user_id}/session`
사용자 세션 목록 조회

**Headers:**
- `Authorization: Bearer <token>`

**Response:**
```json
{
  "items": [
    {
      "session_id": "dashboard-1"
    }
  ]
}
```

#### `GET /stream/{user_id}/{session_id}`
실시간 SVG 스트림 (인증 불필요)

**Response:**
```
Content-Type: multipart/x-mixed-replace; boundary=frame

--frame
Content-Type: image/svg+xml

<svg>...</svg>
--frame
...
```

### 공용 세션 API (하위 호환)

기존 인증 없는 세션은 계속 지원됩니다:

- `POST /api/session`
- `PUT /api/session/{session_id}`
- `GET /stream/{session_id}`

## 프로젝트 구조

```
realtime-svg/
├── crates/
│   ├── common/          # 공통 로직
│   │   ├── jwt.rs       # JWT 생성/검증
│   │   ├── jwk.rs       # JWK 관리 및 캐싱
│   │   ├── auth.rs      # AuthenticatedUser extractor
│   │   ├── state.rs     # AppState (Redis, JwkCache)
│   │   └── session_data.rs  # SessionData 모델
│   ├── backend/         # HTTP 서버
│   │   ├── route/
│   │   │   ├── api/
│   │   │   │   ├── auth/    # JWT 발급 API
│   │   │   │   ├── user/    # 사용자 세션 API
│   │   │   │   └── session/ # 공용 세션 API
│   │   │   └── stream/      # 스트림 엔드포인트
│   │   └── tests/       # 통합 테스트
│   └── frontend/        # Yew WASM 웹 클라이언트
│       ├── src/
│       │   ├── components/  # UI 컴포넌트
│       │   ├── api/         # API 클라이언트
│       │   └── auth/        # JWT 인증 로직
│       └── styles.css   # 다크 테마 스타일
└── specs/               # 기능 명세서
    └── 002-user-session-auth/
        ├── spec.md      # 요구사항 명세
        ├── plan.md      # 기술 설계
        ├── tasks.md     # 구현 태스크
        └── contracts/   # API 계약
```

## 기술 스택

### 백엔드
- **언어:** Rust 2021
- **웹 프레임워크:** Axum 0.8
- **템플릿:** Tera
- **인증:** jsonwebtoken 10 (RS256)
- **스토리지:** Redis 7+
- **빌드:** Cargo workspace

### 프론트엔드
- **프레임워크:** Yew 0.21 (Rust WASM)
- **라우팅:** yew-router 0.18
- **HTTP 클라이언트:** gloo-net 0.4
- **빌드:** Trunk
- **스타일:** 커스텀 CSS (다크 테마)

## 보안

### JWT 키 관리

- **알고리즘:** RSA-2048 with SHA-256 (RS256)
- **키 생성:** 서버 시작 시 자동 생성 (없는 경우)
- **저장소:** Redis (`rsa:private_pem`, `rsa:public_pem`)
- **캐싱:** 메모리 캐시 (OnceCell)로 성능 최적화
- **원자성:** Redis `SET NX`로 동시 생성 방지

### 권한 모델

- 세션 **읽기**: 인증 불필요 (공개)
- 세션 **쓰기**: JWT 인증 + 소유자 검증 필요
- 토큰 검증 실패: 401 Unauthorized
- 소유자 불일치: 403 Forbidden

## 테스트

```bash
# 전체 테스트 실행
cargo test

# 특정 테스트만 실행
cargo test jwt_flow
cargo test user_session

# 통합 테스트 (Redis 필요)
cargo test --test us1_session_creation_test
cargo test --test us2_authorization_test
```

**테스트 현황:** ✅ 26개 테스트 통과

## 성능

- **JWT 검증:** < 50ms (메모리 캐시 사용)
- **토큰 발급:** < 1초
- **JWK 초기화:** < 5초
- **세션 용량:** 사용자당 100+ 세션 지원

## 라이선스

MIT License

## 기여

Issue 및 PR은 언제나 환영합니다!

## 참고 문서

- [기능 명세서](./specs/002-user-session-auth/spec.md)
- [빠른 시작 가이드](./specs/002-user-session-auth/quickstart.md)
- [API 계약](./specs/002-user-session-auth/contracts/)
