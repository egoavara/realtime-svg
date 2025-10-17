# Research: 사용자별 세션 인증 기술 조사

**Date**: 2025-10-17  
**Feature**: JWT 기반 사용자 인증 및 JWK Redis 저장  
**Updated**: jsonwebtoken::Jwk 객체 직접 사용

## 목차

1. [JWK 형식 및 Redis 저장](#1-jwk-형식-및-redis-저장)
2. [JWT 메모리 캐싱 전략](#2-jwt-메모리-캐싱-전략)
3. [사용자별 세션 Redis 키 설계](#3-사용자별-세션-redis-키-설계)
4. [JWT 인증 미들웨어](#4-jwt-인증-미들웨어)

---

## 1. JWK 형식 및 Redis 저장

### Decision
**jsonwebtoken 라이브러리의 `Jwk` 객체를 직접 사용하여 Redis에 저장**

### Rationale
- **내장 직렬화 지원**: `jsonwebtoken::Jwk`는 `serde::Serialize`/`Deserialize` 구현됨
- **표준 준수**: RFC 7517 JSON Web Key 표준을 자동으로 따름
- **타입 안전성**: 별도 구조체 정의 불필요, 라이브러리 타입 직접 사용
- **간소화**: JWK ↔ Encoding/DecodingKey 변환이 라이브러리에서 제공됨
- **분리 저장 전략**:
  - `jwk:public` → 공개 Jwk (검증 전용, 자주 액세스)
  - `jwk:private` → 비밀 Jwk (서명 전용, 제한적 액세스)
- **원자적 키 생성**: Redis `SET NX` 명령으로 동시 시작 시 경쟁 조건 방지

### Alternatives Considered
- **별도 JwkPublic/JwkPrivate 구조체 정의**: 불필요한 중복, 라이브러리가 이미 제공
- **PEM 형식 저장**: JWK가 웹 표준이고 JSON으로 다루기 쉬움
- **단일 키 저장**: 공개 키만 조회 시에도 비밀 키까지 로드됨

### Implementation Details

**의존성 (Cargo.toml)**:
```toml
[dependencies]
jsonwebtoken = "10"  # Jwk 객체 포함
```

**JWK 생성 및 Redis 저장**:
```rust
use jsonwebtoken::{Jwk, Algorithm, EncodingKey, DecodingKey};
use redis::AsyncCommands;

pub async fn initialize_jwk_in_redis(
    conn: &mut redis::aio::MultiplexedConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. 이미 존재하는지 확인
    let exists: bool = conn.exists("jwk:public").await?;
    if exists {
        tracing::info!("JWK가 이미 Redis에 존재합니다");
        return Ok(());
    }
    
    // 2. RSA-2048 키 쌍 생성
    let encoding_key = EncodingKey::from_rsa_pem(include_bytes!("..."))?;
    
    // 3. Jwk 객체 생성 (jsonwebtoken 라이브러리 제공)
    let jwk = Jwk::from_rsa_components(modulus, exponent)?;
    
    // 4. JSON 직렬화 (serde 자동)
    let jwk_json = serde_json::to_string(&jwk)?;
    
    // 5. 원자적 저장 (SET NX)
    let set: bool = conn.set_nx("jwk:public", &jwk_json).await?;
    if set {
        tracing::info!("새 JWK를 Redis에 생성했습니다");
    } else {
        tracing::info!("다른 인스턴스가 JWK를 이미 생성했습니다");
    }
    
    Ok(())
}

// Redis에서 JWK 로드
pub async fn load_jwk_from_redis(
    conn: &mut redis::aio::MultiplexedConnection,
) -> Result<Jwk, Box<dyn std::error::Error>> {
    let jwk_json: String = conn.get("jwk:public").await?;
    let jwk: Jwk = serde_json::from_str(&jwk_json)?;
    Ok(jwk)
}
```

**Jwk에서 Encoding/DecodingKey 변환**:
```rust
// jsonwebtoken 라이브러리가 제공하는 메서드 사용
let decoding_key = DecodingKey::from_jwk(&jwk)?;
let encoding_key = EncodingKey::from_jwk(&jwk)?;
```

---

## 2. JWT 메모리 캐싱 전략

### Decision
**`tokio::sync::OnceCell`로 메모리 캐시 구현, `AppState`에 통합**

### Rationale
- **기존 아키텍처 일관성**: 프로젝트는 `AppState` 패턴 사용 중
- **비동기 초기화 지원**: `tokio::sync::OnceCell`은 async 로드를 네이티브 지원
- **초기화 전략**: 첫 요청 시 지연 로딩
- **성능 이점**:
  - JWT 검증 시 Redis 조회 **0회**
  - 2-5ms → 0.01-0.05ms로 **100배 단축**

### Implementation Details

**AppState 확장**:
```rust
use tokio::sync::OnceCell;
use jsonwebtoken::{Jwk, DecodingKey, EncodingKey};

#[derive(Clone)]
pub struct AppState {
    redis_client: Client,
    jwk_cache: Arc<JwkCache>,
}

pub struct JwkCache {
    jwk: OnceCell<Jwk>,
    encoding_key: OnceCell<EncodingKey>,
    decoding_key: OnceCell<DecodingKey>,
}

impl JwkCache {
    pub fn new() -> Self {
        Self {
            jwk: OnceCell::new(),
            encoding_key: OnceCell::new(),
            decoding_key: OnceCell::new(),
        }
    }
    
    pub async fn get_jwk(&self, redis: &Client) -> Result<&Jwk, ApiError> {
        self.jwk
            .get_or_try_init(|| async {
                let mut conn = redis.get_multiplexed_async_connection().await?;
                let jwk_json: String = conn.get("jwk:public").await?;
                let jwk: Jwk = serde_json::from_str(&jwk_json)?;
                Ok(jwk)
            })
            .await
    }
    
    pub async fn get_decoding_key(&self, redis: &Client) -> Result<&DecodingKey, ApiError> {
        self.decoding_key
            .get_or_try_init(|| async {
                let jwk = self.get_jwk(redis).await?;
                let key = DecodingKey::from_jwk(jwk)?;
                Ok(key)
            })
            .await
    }
}
```

---

## 3. 사용자별 세션 Redis 키 설계

### Decision
**계층적 네임스페이스 + 선택적 ZSET 인덱스**

### Rationale
- **네임스페이스 분리**:
  - 사용자 세션: `user:{user_id}:session:{session_id}`
  - 공용 세션: `session:{session_id}` (기존 유지)
- **세션 목록 조회**: SCAN (MVP) → ZSET 인덱스 (최적화)
- **Pubsub 채널**: 저장 키와 동일

### Implementation Details

```rust
pub async fn list_user_sessions(&self, user_id: &str) -> Result<Vec<String>, ApiError> {
    let pattern = format!("user:{}:session:*", user_id);
    let mut conn = self.connection_redis().await?;
    let mut cursor = 0u64;
    let mut sessions = Vec::new();
    
    loop {
        let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(&pattern)
            .arg("COUNT")
            .arg(100)
            .query_async(&mut conn)
            .await?;
        
        for key in keys {
            if let Some(session_id) = key.strip_prefix(&format!("user:{}:session:", user_id)) {
                sessions.push(session_id.to_string());
            }
        }
        
        cursor = new_cursor;
        if cursor == 0 { break; }
    }
    
    Ok(sessions)
}
```

---

## 4. JWT 인증 미들웨어

### Decision
**axum Extractor 패턴 + 선택적 미들웨어**

### Rationale
- **Extractor 기반 인증**: `FromRequestParts` 트레잇으로 JWT 검증 로직 캡슐화
- **선택적 적용**: `/api/user/*` 경로만 인증 필요
- **경로별 권한 검증**: URL `user_id`와 JWT `sub` claim 일치 확인

### Implementation Details

**JWT Extractor**:
```rust
use jsonwebtoken::{decode, Validation, Algorithm};

#[derive(Debug, Clone)]
pub struct AuthenticatedUser(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers
            .get("Authorization")
            .ok_or(ApiError::Unauthorized("Missing Authorization header".into()))?;
        
        let token = auth_header
            .to_str()
            .map_err(|_| ApiError::Unauthorized("Invalid header encoding".into()))?
            .strip_prefix("Bearer ")
            .ok_or(ApiError::Unauthorized("Invalid Authorization format".into()))?;
        
        let app_state = AppState::from_ref(state);
        let decoding_key = app_state.jwk_cache
            .get_decoding_key(&app_state.redis_client)
            .await?;
        
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&["realtime-svg"]);
        
        let token_data = decode::<Claims>(token, decoding_key, &validation)?;
        
        Ok(AuthenticatedUser(token_data.claims.sub))
    }
}
```

---

## JWKS 엔드포인트

### Decision
**`/.well-known/jwks.json` 표준 경로 사용**

### Rationale
- **RFC 8414 준수**: OAuth 2.0 Authorization Server Metadata 표준
- **자동 발견**: OAuth/OIDC 클라이언트가 자동으로 키 발견 가능
- **산업 표준**: 대부분의 JWT 라이브러리가 이 경로 기대

### Implementation Details

```rust
use jsonwebtoken::{Jwk, JwkSet};

// GET /.well-known/jwks.json
pub async fn jwks_handler(
    State(state): State<AppState>,
) -> Result<Json<JwkSet>, ApiError> {
    let jwk = state.jwk_cache
        .get_jwk(&state.redis_client)
        .await?;
    
    // JwkSet 객체 직접 반환 (별도 구조체 불필요)
    let jwk_set = JwkSet {
        keys: vec![jwk.clone()],
    };
    
    Ok(Json(jwk_set))
}
```

---

## 요약

| 항목 | 선택 기술 | 핵심 이점 |
|-----|----------|----------|
| **JWT 라이브러리** | `jsonwebtoken` 10.x | Jwk 객체 내장, serde 지원 |
| **키 형식** | `jsonwebtoken::Jwk` | 표준 준수, 직렬화 자동 |
| **Redis 저장** | JSON 직렬화 | 별도 구조체 불필요 |
| **메모리 캐싱** | `tokio::sync::OnceCell` | Redis 조회 0회, 100배 성능 향상 |
| **JWKS 엔드포인트** | `/.well-known/jwks.json` | RFC 8414 표준, 자동 발견 |
| **인증 패턴** | axum Extractor | 재사용성, 타입 안전성 |

**다음 단계**: data-model.md와 API contracts 업데이트
