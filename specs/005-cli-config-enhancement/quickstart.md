# Quickstart: CLI Configuration Enhancement

**Feature**: 005-cli-config-enhancement  
**Date**: 2025-10-18  
**For**: 개발자 및 운영자

이 가이드는 새로운 계층적 설정 시스템을 빠르게 시작하는 방법을 설명한다.

---

## 1. 기본 사용법 (설정 파일 없이)

가장 간단한 방법은 환경 변수만 사용하는 것이다:

```bash
# 환경 변수 설정
export REDIS_URL="redis://localhost:6379/"
export PORT=8080
export HOST="0.0.0.0"
export LOG_LEVEL="debug"

# 서버 실행
./backend
```

**결과**: 환경 변수로 설정이 적용되고 서버가 시작된다.

---

## 2. .env 파일 사용 (로컬 개발)

프로젝트 루트에 `.env` 파일 생성:

```bash
# .env
REDIS_URL=redis://127.0.0.1:6379/
PORT=3000
HOST=127.0.0.1
LOG_LEVEL=info
```

서버 실행:
```bash
./backend
```

**결과**: `.env` 파일이 자동으로 로드되어 환경 변수로 설정된다.

---

## 3. 설정 파일 사용 (프로덕션)

바이너리와 같은 디렉토리에 `config.yaml` 생성:

```yaml
# config.yaml
redis_url: redis://prod-redis:6379/
host: 0.0.0.0
port: 8080
log_level: warn
```

서버 실행:
```bash
./backend
```

**결과**: `config.yaml`이 자동으로 읽혀 설정이 적용된다.

---

## 4. 커스텀 설정 파일 경로

다른 경로의 설정 파일 사용:

```bash
./backend --config /etc/myapp/production.yaml
```

**결과**: 지정된 경로의 설정 파일이 로드된다.

---

## 5. 우선순위 오버라이드

설정 파일, 환경 변수, CLI 옵션을 모두 혼합:

```yaml
# config.yaml
redis_url: redis://config-redis:6379/
port: 3000
```

```bash
# 환경 변수 설정
export PORT=8080

# CLI 옵션으로 실행
./backend --port 9000 --log-level debug
```

**결과**:
- `redis_url`: config.yaml에서 (`redis://config-redis:6379/`)
- `port`: CLI 옵션에서 (`9000`, ENV와 config.yaml보다 우선)
- `log_level`: CLI 옵션에서 (`debug`)

**우선순위**: CLI > ENV > Config > Default

---

## 6. 도움말 확인

사용 가능한 모든 옵션 확인:

```bash
./backend --help
```

**출력 예시**:
```
Usage: backend [OPTIONS]

Options:
      --redis-url <REDIS_URL>    Redis 서버 URL [env: REDIS_URL=] [default: redis://127.0.0.1/]
      --host <HOST>              서버 바인딩 주소 [env: HOST=] [default: 127.0.0.1]
      --port <PORT>              서버 포트 [env: PORT=] [default: 3000]
      --log-level <LOG_LEVEL>    로깅 레벨 [env: LOG_LEVEL=] [default: info]
      --config <CONFIG>          설정 파일 경로 [default: config.yaml]
  -h, --help                     Print help
```

---

## 7. 설정 검증 및 디버깅

### 잘못된 설정 값

```bash
./backend --port 999999
```

**출력**:
```
Error: 설정 검증 실패

Caused by:
    포트는 1-65535 범위여야 합니다: 999999
```

### 설정 파일 문법 오류

```yaml
# config.yaml (잘못된 문법)
redis_url redis://localhost/
```

```bash
./backend
```

**출력**:
```
Error: 설정 파일 로딩 실패

Caused by:
    invalid YAML at line 1, column 11: unexpected character
```

### 최종 설정 확인

서버 시작 시 로그에서 적용된 설정 확인:

```
INFO 서버 설정: Config { redis_url: "***REDACTED***", host: "0.0.0.0", port: 8080, log_level: "info" }
```

**Note**: `redis_url`은 민감 정보로 자동 마스킹됨.

---

## 8. 배포 시나리오별 가이드

### 시나리오 A: Docker 컨테이너

`Dockerfile`:
```dockerfile
FROM rust:1.90 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/backend /usr/local/bin/
CMD ["backend"]
```

`docker run`:
```bash
docker run -e REDIS_URL=redis://redis-host:6379/ \
           -e PORT=8080 \
           -e HOST=0.0.0.0 \
           myapp/backend
```

**Why**: 환경 변수를 통한 설정이 컨테이너 표준 패턴.

---

### 시나리오 B: Kubernetes ConfigMap + Secret

`configmap.yaml`:
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: backend-config
data:
  config.yaml: |
    host: 0.0.0.0
    port: 8080
    log_level: info
```

`secret.yaml`:
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: backend-secret
stringData:
  REDIS_URL: redis://redis-svc:6379/
```

`deployment.yaml`:
```yaml
apiVersion: apps/v1
kind: Deployment
spec:
  template:
    spec:
      containers:
      - name: backend
        env:
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: backend-secret
              key: REDIS_URL
        volumeMounts:
        - name: config
          mountPath: /etc/backend
      volumes:
      - name: config
        configMap:
          name: backend-config
      command: ["backend", "--config", "/etc/backend/config.yaml"]
```

**Why**: 민감 정보(REDIS_URL)는 Secret, 일반 설정은 ConfigMap으로 분리.

---

### 시나리오 C: Systemd 서비스 (bare metal)

`/etc/systemd/system/backend.service`:
```ini
[Unit]
Description=Backend Server
After=network.target

[Service]
Type=simple
User=backend
WorkingDirectory=/opt/backend
ExecStart=/opt/backend/backend --config /etc/backend/config.yaml
Restart=on-failure
Environment="REDIS_URL=redis://localhost:6379/"

[Install]
WantedBy=multi-user.target
```

`/etc/backend/config.yaml`:
```yaml
host: 0.0.0.0
port: 8080
log_level: warn
```

**Why**: 설정 파일로 일반 설정, 환경 변수로 민감 정보 관리.

---

## 9. 트러블슈팅

### 문제: 설정이 적용되지 않음

**체크리스트**:
1. 우선순위 확인: CLI > ENV > Config
2. 환경 변수 이름 확인 (대소문자 구분: `REDIS_URL`)
3. YAML 필드 이름 확인 (snake_case: `redis_url`)
4. 설정 파일 경로 확인 (기본: 바이너리 디렉토리의 config.yaml)

**디버깅**:
```bash
# 환경 변수 확인
printenv | grep -E 'REDIS_URL|PORT|HOST|LOG_LEVEL'

# 설정 파일 경로 확인
./backend --config /tmp/debug.yaml  # 절대 경로 사용

# 로그 레벨 올리기
./backend --log-level trace
```

---

### 문제: "설정 파일 로딩 실패"

**원인**: YAML 문법 오류 또는 타입 불일치

**해결**:
1. YAML 검증기로 확인: https://www.yamllint.com/
2. 타입 확인: `port`는 숫자, `redis_url`은 문자열
3. 주석 사용 시 `#` 앞에 공백 필요

**예시 (올바른 YAML)**:
```yaml
# 주석
redis_url: redis://localhost/  # 인라인 주석
port: 3000                      # 숫자는 따옴표 없음
```

---

### 문제: 민감 정보가 로그에 노출됨

**확인**:
```bash
./backend 2>&1 | grep redis_url
```

**예상 출력**:
```
INFO 서버 설정: Config { redis_url: "***REDACTED***", ... }
```

**만약 실제 값이 보인다면**: Config 구조체의 Debug impl이 잘못 구현됨 (버그 리포트).

---

## 10. 예제 파일

### config.yaml.example (체크인)
```yaml
# Backend Server Configuration
# 이 파일을 config.yaml로 복사하여 사용하세요

redis_url: redis://127.0.0.1:6379/
host: 127.0.0.1
port: 3000
log_level: info
```

### .env.example (체크인)
```bash
# Environment Variables
# 이 파일을 .env로 복사하여 사용하세요

REDIS_URL=redis://127.0.0.1:6379/
PORT=3000
HOST=127.0.0.1
LOG_LEVEL=info
```

---

## 11. 마이그레이션 가이드

### 기존 방식 (환경 변수만)
```bash
export REDIS_URL=redis://localhost/
./backend
```

### 새 방식 (호환성 유지)
```bash
# 아무 변경 없이 그대로 동작
export REDIS_URL=redis://localhost/
./backend
```

**Breaking Changes**: 없음 (100% 하위 호환).

### 점진적 마이그레이션

**Step 1**: .env 파일 생성 (개발 환경)
```bash
cp .env.example .env
# .env 파일 수정
```

**Step 2**: config.yaml 생성 (프로덕션)
```bash
cp config.yaml.example config.yaml
# config.yaml 파일 수정
```

**Step 3**: systemd/Dockerfile 환경 변수 제거 (선택적)
```bash
# 환경 변수 대신 설정 파일 사용
# ENV를 민감 정보만 남기고 제거
```

---

## 12. 추가 리소스

- **Figment 문서**: https://docs.rs/figment/
- **Clap 문서**: https://docs.rs/clap/
- **12-factor config**: https://12factor.net/config
- **YAML 사양**: https://yaml.org/spec/1.2/spec.html

**질문/이슈**: GitHub Issues에 리포트
