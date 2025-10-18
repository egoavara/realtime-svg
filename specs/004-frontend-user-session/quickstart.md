# Quickstart Guide: 프론트엔드 유저 세션 UI 통합

**Feature**: 004-frontend-user-session  
**Date**: 2025-10-18  
**Purpose**: 로컬 개발 환경에서 프론트엔드 유저 세션 기능 테스트

## 전제 조건

### 시스템 요구사항
- **Rust**: 1.75 이상 (2021 Edition)
- **Cargo**: Rust 설치 시 포함
- **브라우저**: Chrome 90+, Firefox 88+, Safari 14+ (WASM 지원)

### 설치 필요 도구

#### 1. Trunk (WASM 개발 서버)
```bash
cargo install trunk
```

#### 2. WASM 타겟
```bash
rustup target add wasm32-unknown-unknown
```

#### 3. Redis (백엔드 세션 저장소)
```bash
# macOS
brew install redis
brew services start redis

# Ubuntu/Debian
sudo apt-get install redis-server
sudo systemctl start redis

# Docker
docker run -d -p 6379:6379 redis:7-alpine
```

#### 4. 의존성 확인
```bash
cd /workspaces/realtime-svg
cargo check --workspace
```

---

## 백엔드 실행 (필수)

프론트엔드는 백엔드 API에 의존하므로 먼저 백엔드를 실행해야 합니다.

### 터미널 1: 백엔드 서버 실행

```bash
cd /workspaces/realtime-svg/crates/backend
cargo run --release
```

**예상 출력**:
```
INFO realtime_svg_backend: Starting server on 0.0.0.0:3000
INFO realtime_svg_backend: JWK initialized in Redis
INFO realtime_svg_backend: Server ready
```

**확인**:
```bash
curl http://localhost:3000/api/session
# 404 응답 (정상, session_id 없이 호출했으므로)

curl http://localhost:3000/.well-known/jwks.json
# {"keys": [{"kty": "RSA", ...}]} 응답 (정상)
```

---

## 프론트엔드 실행

### 터미널 2: 프론트엔드 개발 서버 실행

```bash
cd /workspaces/realtime-svg/crates/frontend
trunk serve --open
```

**trunk serve 옵션**:
- `--open`: 브라우저 자동 열기
- `--port 8080`: 포트 지정 (기본값: 8080)
- `--address 0.0.0.0`: 외부 접근 허용

**예상 출력**:
```
INFO trunk::serve: serving static assets at -> /
INFO trunk::serve: spawned server at http://127.0.0.1:8080
INFO trunk::build: building app
INFO trunk::build: finished in 2.3s
```

**브라우저 자동 열림**: `http://127.0.0.1:8080`

---

## 기능 테스트 시나리오

### 시나리오 1: JWT 토큰 발급 및 로그인

1. **홈 페이지 접근** (`http://127.0.0.1:8080`)
   - "비로그인 상태" 표시 확인

2. **토큰 발급 폼 입력**
   - user_id 입력란에 `alice` 입력
   - "토큰 발급" 버튼 클릭

3. **로그인 상태 확인**
   - 헤더에 "안녕하세요, alice" 표시
   - "로그아웃" 버튼 표시

4. **브라우저 개발자 도구로 확인**
   - F12 → Application → Local Storage → `http://127.0.0.1:8080`
   - `jwt_token` 키에 토큰 저장 확인

5. **페이지 새로고침**
   - 로그인 상태 유지 확인 (localStorage에서 토큰 자동 로드)

---

### 시나리오 2: 유저별 세션 생성 및 수정

1. **세션 생성 페이지 이동**
   - 홈 페이지에서 "유저별 세션 만들기" 선택

2. **세션 정보 입력**
   - session_id: `test-dashboard`
   - template (기본값 유지):
     ```svg
     <svg xmlns="http://www.w3.org/2000/svg" width="600" height="320">
       <text x="50%" y="50%" text-anchor="middle">
         {{ headline | default(value="realtime-svg") }}
       </text>
     </svg>
     ```
   - "세션 생성" 버튼 클릭

3. **상세 페이지 리다이렉트 확인**
   - URL: `http://127.0.0.1:8080/session/alice/test-dashboard`
   - 스트림 미리보기에 SVG 표시 확인
   - 템플릿 읽기 전용 표시 확인

4. **파라미터 수정**
   - args JSON 편집:
     ```json
     {
       "headline": "Hello World"
     }
     ```
   - "세션 파라미터 업데이트" 버튼 클릭

5. **실시간 스트림 갱신 확인**
   - 스트림 미리보기가 "Hello World"로 변경됨
   - "업데이트가 완료되었습니다" 메시지 표시

---

### 시나리오 3: 권한 검증

1. **다른 사용자로 로그인**
   - "로그아웃" 버튼 클릭
   - user_id: `bob` 입력하여 새 토큰 발급

2. **alice의 세션 접근**
   - URL 직접 입력: `http://127.0.0.1:8080/session/alice/test-dashboard`
   - 세션 정보는 정상 표시 (읽기는 공개)

3. **alice의 세션 수정 시도**
   - args 편집 후 "업데이트" 클릭
   - **예상 결과**: "권한이 없습니다. 세션 소유자만 수정할 수 있습니다" 에러 메시지
   - HTTP 403 Forbidden 응답

---

### 시나리오 4: 세션 목록 조회

1. **alice로 로그인**
   - bob 로그아웃 → alice 토큰 재발급

2. **여러 세션 생성**
   - `dashboard-1`
   - `widget-status`
   - `analytics-chart`

3. **세션 목록 페이지 이동**
   - "내 세션 목록" 버튼 클릭
   - URL: `http://127.0.0.1:8080/my-sessions`

4. **목록 확인**
   - 3개 세션 카드 표시
   - 각 카드에 session_id, 생성 시간, 남은 TTL 표시

5. **세션 클릭**
   - `dashboard-1` 카드 클릭
   - 상세 페이지로 이동: `/session/alice/dashboard-1`

---

### 시나리오 5: 공용 세션 호환성 (하위 호환성)

1. **로그아웃**
   - alice 로그아웃하여 비로그인 상태로 전환

2. **공용 세션 생성**
   - "공용 세션 만들기" 선택
   - session_id: `public-demo`
   - template 입력 후 "세션 생성" 클릭

3. **공용 세션 URL 확인**
   - URL: `http://127.0.0.1:8080/session/public-demo`
   - 유저별 세션과 달리 user_id 세그먼트 없음

4. **공용 세션 수정 (비로그인 상태)**
   - args 편집 후 "업데이트" 클릭
   - **예상 결과**: 수정 성공 (인증 불필요)

5. **alice로 로그인 후 공용 세션 수정**
   - alice 토큰 발급 후 동일한 공용 세션 수정
   - **예상 결과**: 수정 성공 (공용 세션은 누구나 수정 가능)

---

### 시나리오 6: 토큰 만료 처리

**수동 만료 시뮬레이션** (개발 환경):

1. **브라우저 개발자 도구에서 토큰 수정**
   - F12 → Application → Local Storage
   - `jwt_token` 값을 잘못된 토큰으로 변경

2. **유저별 세션 수정 시도**
   - args 편집 후 "업데이트" 클릭
   - **예상 결과**: 401 Unauthorized 응답
   - "토큰이 만료되었습니다. 다시 로그인하세요" 메시지
   - localStorage에서 토큰 자동 삭제
   - AuthState → Anonymous

3. **자동 로그아웃 확인**
   - 헤더에 "비로그인 상태" 표시
   - 유저별 세션 생성/수정 버튼 비활성화

---

## 디버깅 팁

### 브라우저 콘솔 로그 확인

```javascript
// F12 → Console
// WASM 로그 출력 예시:
[INFO] Token loaded from localStorage
[INFO] Valid token for user: alice
[WARN] 401 Unauthorized - token expired
[ERROR] Failed to parse token: Invalid JWT format
```

### 네트워크 요청 확인

F12 → Network 탭:
- `POST /api/auth/token` → 토큰 발급 요청
- `POST /api/user/alice/session` → 유저별 세션 생성
- `PUT /api/user/alice/session/test-1` → 세션 수정
- `GET /stream/alice/test-1` → SVG 스트림 (multipart/x-mixed-replace)

**Authorization 헤더 확인**:
- 유저별 세션 API: `Authorization: Bearer eyJ...`
- 공용 세션 API: 헤더 없음

### Redis 데이터 확인

```bash
redis-cli

# JWK 확인
GET jwk:public

# 유저별 세션 확인
GET user:alice:session:test-dashboard

# 공용 세션 확인
GET session:public-demo

# 세션 목록 (패턴 매칭)
KEYS user:alice:session:*
```

### 빌드 오류 해결

**WASM 빌드 실패**:
```bash
# 캐시 삭제
cargo clean
rm -rf target/

# 재빌드
trunk build
```

**의존성 충돌**:
```bash
cargo update
cargo check
```

---

## 성능 확인

### WASM 번들 크기

```bash
trunk build --release
ls -lh dist/
```

**예상 크기**:
- `frontend_bg.wasm`: ~300-500 KB (압축 전)
- `frontend.js`: ~50 KB

### 페이지 로드 시간

F12 → Performance:
- WASM 로드 + 초기화: < 1초
- localStorage 읽기: < 10ms
- JWT 디코딩: < 1ms

### API 응답 시간

F12 → Network → Timing:
- `POST /api/auth/token`: < 100ms
- `GET /api/user/{user_id}/sessions`: < 200ms

---

## 트러블슈팅

### localStorage 비활성화 경고

**증상**: "브라우저 설정을 확인하세요" 메시지

**원인**: 시크릿 모드 또는 쿠키 차단 설정

**해결**:
1. 일반 브라우저 모드로 전환
2. 설정 → 개인정보 → 쿠키 허용

**폴백**: 공용 세션은 계속 사용 가능

### CORS 오류

**증상**: `Access to fetch blocked by CORS policy`

**원인**: 백엔드 CORS 헤더 누락

**해결**: 백엔드 코드에서 CORS 미들웨어 확인
```rust
// crates/backend/src/main.rs
use tower_http::cors::CorsLayer;

let app = Router::new()
    .layer(CorsLayer::permissive());
```

### 401 반복 발생

**증상**: 로그인 직후 401 응답

**원인**: 백엔드 JWK 불일치

**해결**:
```bash
# Redis JWK 삭제 후 백엔드 재시작
redis-cli DEL jwk:public jwk:private

# 백엔드 재시작
cd crates/backend
cargo run
```

---

## 다음 단계

1. ✅ 로컬 환경 구성 완료
2. ✅ 기능 테스트 통과
3. 🔄 `/speckit.tasks` 실행하여 작업 목록 생성
4. 🔄 구현 시작 (tasks.md 기준)
5. 🔄 단위 테스트 작성 (`wasm-bindgen-test`)
6. 🔄 통합 테스트 (E2E)

**문서 업데이트**: 구현 완료 후 이 quickstart.md를 실제 코드 경로로 업데이트
