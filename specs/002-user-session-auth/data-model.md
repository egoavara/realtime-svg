# Data Model: 사용자별 세션 인증

**Feature**: 002-user-session-auth  
**Date**: 2025-10-17

## 개요

이 문서는 JWT 기반 사용자 인증 및 세션 관리를 위한 데이터 모델을 정의합니다.

## 핵심 엔티티

### 1. JWK (JSON Web Key)

JWT 토큰 서명 및 검증에 사용되는 RSA-2048 키 쌍. `jsonwebtoken` 라이브러리의 `Jwk` 객체를 직접 사용.

#### Jwk 객체 (jsonwebtoken 라이브러리 제공)
```rust
// jsonwebtoken::Jwk 사용 (serde::Serialize/Deserialize 자동 지원)
use jsonwebtoken::Jwk;

// 공개 키 예시 (RFC 7517 표준)
{
    "kty": "RSA",
    "n": "xGOr-H7A-wiuVDjc...",  // Modulus (base64url)
    "e": "AQAB",                   // Exponent (base64url)
    "alg": "RS256",
    "use": "sig"
}

// 비밀 키는 추가로 d, p, q 포함
```

**저장 위치**: 
- 공개 키: Redis `jwk:public` (JSON 직렬화)
- 비밀 키: Redis `jwk:private` (JSON 직렬화)

**TTL**: 영구 (TTL 없음)  
**용도**: 
- 공개 키: JWT 검증, `/.well-known/jwks.json` 엔드포인트에서 `JwkSet`으로 노출
- 비밀 키: JWT 서명 (외부 노출 금지)

**타입 안전성**: 
- `Jwk` 객체: 개별 키 표현
- `JwkSet` 객체: JWKS 엔드포인트 응답 (`{"keys": [...]}`)

#### 상태 전이
```
[없음] --시스템 시작--> [생성] --Redis 저장(SET NX)--> [영구 저장]
                                    ↓
                            [메모리 캐시 로드]
```

#### 검증 규칙
- `kty`는 반드시 "RSA"
- `n`, `e`는 base64url 인코딩된 값
- 비밀 키는 `d`, `p`, `q` 필수
- 공개 키와 비밀 키의 `n`, `e`는 동일해야 함

---

### 2. JWT Token

사용자 인증 정보를 담은 JSON Web Token.

#### Claims 구조
```rust
struct Claims {
    sub: String,    // Subject (사용자 ID)
    exp: usize,     // Expiration (만료 시간, Unix timestamp)
    iat: usize,     // Issued At (발급 시간, Unix timestamp)
    iss: String,    // Issuer (발급자, "realtime-svg")
}
```

#### Header
```json
{
  "alg": "RS256",
  "typ": "JWT"
}
```

#### 토큰 형식
```
eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyMTIzIiwiZXhwIjoxNzM0NDgwMDAwLCJpYXQiOjE3MzQ0MDAwMDAsImlzcyI6InJlYWx0aW1lLXN2ZyJ9.signature...
```

#### 수명주기
```
[발급 요청] --> [Claims 생성] --> [JWK로 서명] --> [토큰 반환]
                                                      ↓
                                              [클라이언트 저장]
                                                      ↓
                                          [요청 시 Authorization 헤더]
                                                      ↓
                                          [JWK로 검증] --> [Claims 추출]
                                                      ↓
                                          [만료 시 401 Unauthorized]
```

#### 검증 규칙
- `exp` > 현재 시간 (만료 체크)
- `iss` == "realtime-svg" (발급자 검증)
- 서명이 JWK 공개 키로 검증 가능
- `sub`가 유효한 문자열 (비어있지 않음)

---

### 3. User Session (사용자별 세션)

특정 사용자가 소유한 세션.

#### SessionData 확장
```rust
struct SessionData {
    owner: Option<String>,         // 소유자 user_id (공용 세션은 None)
    template: String,              // Tera 템플릿 문자열
    args: HashMap<String, Value>,  // 템플릿 파라미터
    created_at: DateTime<Utc>,     // 생성 시간
}
```

**저장 위치**: 
- 사용자 세션: Redis `user:{user_id}:session:{session_id}`
- 공용 세션: Redis `session:{session_id}`

**TTL**: 3600초 (1시간)

**Pubsub 채널**:
- 사용자 세션: `user:{user_id}:session:{session_id}`
- 공용 세션: `session:{session_id}`

#### 상태 전이
```
[생성 요청] --인증 확인--> [owner 설정] --Redis 저장--> [활성]
                                                          ↓
                                           [파라미터 업데이트] --Pubsub 브로드캐스트--> [연결된 클라이언트 갱신]
                                                          ↓
                                                   [TTL 만료] --> [삭제]
```

#### 검증 규칙
- 사용자 세션: `owner` 필드 필수, JWT의 `sub`와 일치해야 함
- 공용 세션: `owner` 필드 None
- `template`은 유효한 Tera 템플릿 문법
- `args`는 직렬화 가능한 JSON 객체

---

### 4. User Session Index (선택적 최적화)

사용자별 세션 목록 조회를 위한 보조 인덱스.

#### ZSET 구조
```
Redis Key: user:{user_id}:sessions
Type: ZSET (Sorted Set)
Member: session_id (String)
Score: created_at (Unix timestamp)
```

**목적**: O(log N + M) 복잡도로 세션 목록 조회 (SCAN보다 빠름)

**생성 시점**: 세션 생성/수정 시 자동 업데이트

**TTL**: 가장 오래된 세션의 TTL과 동기화

#### 사용 예시
```redis
# 최신 세션 10개 조회
ZREVRANGE user:alice:sessions 0 9

# 모든 세션 조회 (생성 시간 내림차순)
ZREVRANGE user:alice:sessions 0 -1

# 특정 시간 이후 세션 조회
ZRANGEBYSCORE user:alice:sessions 1734400000 +inf
```

---

### 5. SvgFrame (기존 유지)

렌더링된 SVG 프레임 (변경 없음).

```rust
struct SvgFrame {
    content: String,  // 렌더링된 SVG XML
}
```

**생성**: SessionData의 `template` + `args`를 Tera로 렌더링  
**전송**: Pubsub 채널로 브로드캐스트  
**프로토콜**: `multipart/x-mixed-replace; boundary=frame`

---

## 엔티티 관계도

```
┌─────────────┐
│  JwkPublic  │──────┐
└─────────────┘      │
                     │ 키 쌍
┌─────────────┐      │
│ JwkPrivate  │──────┘
└─────────────┘
      │ 서명/검증
      ↓
┌─────────────┐
│  JWT Token  │
└─────────────┘
      │ 인증
      ↓
┌──────────────────┐        ┌─────────────────┐
│  User Session    │───────→│  SvgFrame       │
│  (SessionData +  │ 렌더링  │  (SVG content)  │
│   owner field)   │        └─────────────────┘
└──────────────────┘
      │
      ↓
┌──────────────────────┐
│ User Session Index   │
│ (ZSET - optional)    │
└──────────────────────┘
```

## Redis 키 스키마 요약

| 키 패턴 | 타입 | 용도 | TTL |
|---------|------|------|-----|
| `jwk:public` | String (JSON) | JWK 공개 키 | 영구 |
| `jwk:private` | String (JSON) | JWK 비밀 키 | 영구 |
| `user:{user_id}:session:{session_id}` | String (JSON) | 사용자 세션 데이터 | 3600s |
| `user:{user_id}:sessions` | ZSET | 세션 목록 인덱스 (선택적) | 3600s |
| `session:{session_id}` | String (JSON) | 공용 세션 데이터 | 3600s |

## 메모리 캐시 구조

```rust
struct JwkCache {
    jwk: OnceCell<Jwk>,                     // JWK 객체 (메모리)
    encoding_key: OnceCell<EncodingKey>,   // JWT 서명용 키 (메모리)
    decoding_key: OnceCell<DecodingKey>,   // JWT 검증용 키 (메모리)
}
```

**초기화**: 첫 JWT 작업 시 Redis에서 `Jwk` 로드 후 캐싱  
**변환**: `DecodingKey::from_jwk(&jwk)`, `EncodingKey::from_jwk(&jwk)`  
**갱신**: 애플리케이션 재시작 전까지 유지  
**메모리 사용량**: ~2KB
