# Quick Start: 사용자별 세션 인증

**Feature**: 002-user-session-auth  
**Date**: 2025-10-17

이 가이드는 JWT 기반 사용자 인증 및 세션 관리 기능을 빠르게 시작하는 방법을 안내합니다.

## 전제 조건

- Rust 1.75+ 설치
- Redis 서버 실행 중 (`redis-server`)
- 프로젝트 빌드 완료 (`cargo build`)

## 시나리오: 사용자별 실시간 대시보드 생성

### 1단계: JWT 토큰 발급

사용자 ID로 JWT 토큰을 발급받습니다.

```bash
curl -X POST http://localhost:3000/api/auth/token \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "alice"
  }'
```

**응답**:
```json
{
  "token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhbGljZSIsImV4cCI6MTczNDQ4MDAwMCwiaWF0IjoxNzM0NDAwMDAwLCJpc3MiOiJyZWFsdGltZS1zdmcifQ.signature..."
}
```

토큰을 환경 변수에 저장:
```bash
export TOKEN="eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."
```

---

### 2단계: 사용자 세션 생성

발급받은 토큰으로 인증하여 세션을 생성합니다.

```bash
curl -X POST http://localhost:3000/api/user/alice/session \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "session_id": "dashboard-1",
    "template": "<svg width=\"200\" height=\"100\"><text x=\"10\" y=\"50\" fill=\"{{color}}\">Status: {{status}}</text></svg>",
    "args": {
      "status": "online",
      "color": "green"
    }
  }'
```

**응답**:
```json
{
  "user_id": "alice",
  "session_id": "dashboard-1"
}
```

---

### 3단계: 스트림에 접근 (인증 불필요)

브라우저나 curl로 실시간 스트림을 확인합니다.

**브라우저**:
```
http://localhost:3000/stream/alice/dashboard-1
```

**curl**:
```bash
curl http://localhost:3000/stream/alice/dashboard-1
```

**스트림 출력**:
```
--frame
Content-Type: image/svg+xml
Content-Length: 123

<svg width="200" height="100"><text x="10" y="50" fill="green">Status: online</text></svg>
--frame
```

---

### 4단계: 세션 파라미터 업데이트

세션 소유자만 파라미터를 수정할 수 있습니다.

```bash
curl -X PUT http://localhost:3000/api/user/alice/session/dashboard-1 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "args": {
      "status": "offline",
      "color": "red"
    }
  }'
```

**응답**: `204 No Content`

스트림에 연결된 모든 클라이언트가 즉시 업데이트된 SVG를 수신합니다:
```xml
<svg width="200" height="100"><text x="10" y="50" fill="red">Status: offline</text></svg>
```

---

### 5단계: 세션 목록 조회

사용자가 생성한 모든 세션을 조회합니다.

```bash
curl -X GET http://localhost:3000/api/user/alice/sessions \
  -H "Authorization: Bearer $TOKEN"
```

**응답**:
```json
{
  "sessions": [
    {
      "session_id": "dashboard-1",
      "created_at": "2025-10-17T10:00:00Z",
      "ttl": 3200
    }
  ]
}
```

---

## 권한 검증 테스트

### 다른 사용자의 세션 수정 시도 (실패 예상)

Bob의 토큰으로 Alice의 세션을 수정하려고 하면 403 Forbidden이 반환됩니다.

```bash
# Bob의 토큰 발급
curl -X POST http://localhost:3000/api/auth/token \
  -H "Content-Type: application/json" \
  -d '{"user_id": "bob"}' \
  | jq -r '.token' > /tmp/bob_token

# Bob이 Alice의 세션 수정 시도
curl -X PUT http://localhost:3000/api/user/alice/session/dashboard-1 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $(cat /tmp/bob_token)" \
  -d '{"args": {"status": "hacked"}}' \
  -i
```

**응답**:
```
HTTP/1.1 403 Forbidden
Content-Type: application/json

{
  "error": "User bob cannot modify sessions of user alice"
}
```

---

## HTML 클라이언트 예제

```html
<!DOCTYPE html>
<html>
<head>
  <title>User Session Stream</title>
</head>
<body>
  <h1>Alice의 대시보드</h1>
  <img src="http://localhost:3000/stream/alice/dashboard-1" alt="Live Dashboard" />
  
  <h2>파라미터 업데이트</h2>
  <button onclick="updateSession('online', 'green')">Online</button>
  <button onclick="updateSession('offline', 'red')">Offline</button>
  
  <script>
    const TOKEN = "YOUR_JWT_TOKEN_HERE";
    
    async function updateSession(status, color) {
      const response = await fetch('http://localhost:3000/api/user/alice/session/dashboard-1', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${TOKEN}`
        },
        body: JSON.stringify({
          args: { status, color }
        })
      });
      
      if (response.status === 204) {
        console.log('세션 업데이트 성공');
      } else {
        const error = await response.json();
        console.error('업데이트 실패:', error);
      }
    }
  </script>
</body>
</html>
```

---

## JWK 공개 키 조회 (고급)

외부 시스템에서 JWT 검증을 위해 공개 키를 조회할 수 있습니다. RFC 8414 표준 경로를 사용합니다.

```bash
curl http://localhost:3000/.well-known/jwks.json
```

**응답**:
```json
{
  "keys": [
    {
      "kty": "RSA",
      "n": "xGOr-H7A-wiuVDjc9FcF8ao9s...",
      "e": "AQAB",
      "alg": "RS256",
      "use": "sig"
    }
  ]
}
```

---

## 문제 해결

### 401 Unauthorized - 토큰 만료
토큰의 기본 만료 시간은 24시간입니다. 만료 시 새 토큰을 발급받으세요.

```bash
# 커스텀 TTL로 토큰 발급 (1시간)
curl -X POST http://localhost:3000/api/auth/token \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "alice",
    "ttl_seconds": 3600
  }'
```

### 403 Forbidden - 권한 없음
토큰의 `sub` claim이 URL의 `user_id`와 일치하는지 확인하세요.

```bash
# 토큰 디코딩 (https://jwt.io 또는 jq 사용)
echo $TOKEN | cut -d. -f2 | base64 -d 2>/dev/null | jq
```

### 404 Not Found - 세션 없음
세션이 TTL(기본 1시간)로 만료되었거나 존재하지 않습니다.

---

## 기존 공용 세션과의 차이점

| 기능 | 공용 세션 | 사용자 세션 |
|-----|----------|-----------|
| **URL 패턴** | `/stream/{session_id}` | `/stream/{user_id}/{session_id}` |
| **생성 API** | `POST /api/session` | `POST /api/user/{user_id}/session` |
| **인증** | 불필요 | JWT 토큰 필요 (쓰기) |
| **수정 권한** | 누구나 가능 | 소유자만 가능 |
| **Redis 키** | `session:{session_id}` | `user:{user_id}:session:{session_id}` |

공용 세션은 하위 호환성을 위해 계속 지원되며, 인증 없이 사용 가능합니다.

---

## 다음 단계

- [data-model.md](./data-model.md) - 데이터 모델 상세 설명
- [contracts/auth-api.yaml](./contracts/auth-api.yaml) - JWT API OpenAPI 스펙
- [contracts/user-session-api.yaml](./contracts/user-session-api.yaml) - 사용자 세션 API 스펙
- [tasks.md](./tasks.md) - 구현 태스크 목록 (Phase 2에서 생성)
