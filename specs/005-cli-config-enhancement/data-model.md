# Data Model: CLI Configuration Enhancement

**Feature**: 005-cli-config-enhancement  
**Date**: 2025-10-18  
**Status**: Complete

이 문서는 설정 시스템에 사용되는 데이터 구조와 상태 전이를 정의한다.

## Core Entities

### 1. Config

**Purpose**: 애플리케이션 실행에 필요한 모든 설정값을 담는 최종 병합된 구조체

**Fields**:
- `redis_url: String` - Redis 서버 연결 URL (예: redis://127.0.0.1/)
- `host: String` - HTTP 서버 바인딩 주소 (예: 127.0.0.1, 0.0.0.0)
- `port: u16` - HTTP 서버 포트 번호 (1-65535)
- `log_level: String` - 로깅 레벨 (예: info, debug, warn, error)

**Validation Rules**:
- `port`: 1 이상 65535 이하
- `redis_url`: "redis://" 또는 "rediss://" 프로토콜로 시작
- `host`: 유효한 IP 주소 또는 호스트명 (빈 문자열 불가)
- `log_level`: tracing EnvFilter 호환 문자열 (잘못된 값은 파싱 시 에러)

**Default Values**:
- `redis_url`: "redis://127.0.0.1/"
- `host`: "127.0.0.1"
- `port`: 3000
- `log_level`: "info"

**Relationships**:
- None (독립적인 설정 구조체, 다른 엔티티와 관계 없음)

**Serialization**:
- `#[derive(Debug, Clone, Serialize, Deserialize)]`
- Custom `Debug` impl: `redis_url` 필드는 민감 정보 마스킹

**Example**:
```yaml
# config.yaml
redis_url: redis://localhost:6379/
host: 0.0.0.0
port: 8080
log_level: debug
```

---

### 2. CliArgs

**Purpose**: 커맨드라인 인자를 파싱한 구조체 (Clap derive API 사용)

**Fields**:
- `redis_url: Option<String>` - `--redis-url` 옵션, ENV: REDIS_URL
- `host: Option<String>` - `--host` 옵션, ENV: HOST
- `port: Option<u16>` - `--port` 옵션, ENV: PORT
- `log_level: Option<String>` - `--log-level` 옵션, ENV: LOG_LEVEL
- `config: PathBuf` - `--config` 옵션, 기본값: "config.yaml"

**Validation Rules**:
- Clap이 자동으로 타입 검증 (예: port는 u16 범위)
- `config`: 파일 존재 여부는 Figment 로딩 시 검증

**Default Values**:
- 모든 Option 필드: None (제공되지 않으면 다음 레이어로 fallback)
- `config`: "config.yaml"

**Relationships**:
- `Config`로 변환됨 (Figment Serialized provider 통해)

**Serialization**:
- `#[derive(Parser, Serialize)]`
- Clap 파싱 후 Figment에 전달

**Example**:
```bash
./backend --redis-url redis://prod:6379/ --port 8080 --config /etc/app/config.yaml
```

---

### 3. ConfigSource (개념적 엔티티)

**Purpose**: 설정값의 출처를 추상화 (구현 시 Figment Provider로 표현)

**Types**:
1. **File Source**: YAML 파일 (`Yaml::file()`)
2. **Environment Source**: 환경 변수 (`Env::prefixed()`)
3. **CLI Source**: 커맨드라인 인자 (`Serialized::defaults()`)

**Precedence** (낮음 → 높음):
1. File Source (config.yaml)
2. Environment Source (.env 또는 시스템 ENV)
3. CLI Source (--옵션)

**Validation Rules**:
- 각 소스는 독립적으로 파싱 가능해야 함
- 파싱 실패 시 명확한 오류 메시지 (소스 이름 + 라인 번호 포함)

**Relationships**:
- 병합되어 최종 `Config` 생성

**State Transitions**:
```
[File YAML] ─┐
             ├─> [Figment Merge] ─> [Config Extract] ─> [Validate] ─> [Final Config]
[ENV Vars]  ─┤
             │
[CLI Args]  ─┘
```

---

## State Transitions

### Configuration Loading Lifecycle

```
┌──────────────┐
│ App Start    │
└──────┬───────┘
       │
       ▼
┌──────────────────┐
│ Load .env file   │ (dotenvy::dotenv())
│ (optional)       │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Parse CLI args   │ (Clap::parse())
└──────┬───────────┘
       │
       ▼
┌──────────────────────────────┐
│ Resolve config file path     │
│ - If relative: exe_dir/path  │
│ - If absolute: path          │
└──────┬───────────────────────┘
       │
       ▼
┌──────────────────────────────┐
│ Merge layers (Figment):      │
│ 1. YAML file (if exists)     │
│ 2. Environment variables     │
│ 3. CLI arguments             │
└──────┬───────────────────────┘
       │
       ▼
┌──────────────────┐
│ Extract to       │
│ Config struct    │ (Figment::extract())
└──────┬───────────┘
       │
       ├─ FAIL ─> [Log parse error + Exit(1)]
       │
       ▼
┌──────────────────┐
│ Validate config  │ (Config::validate())
└──────┬───────────┘
       │
       ├─ FAIL ─> [Log validation error + Exit(1)]
       │
       ▼
┌──────────────────┐
│ Log final config │ (tracing::info!, 민감 정보 마스킹)
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Use config to    │
│ start server     │
└──────────────────┘
```

---

## Field Mapping Across Sources

| Config Field | CLI Option       | Environment Variable | YAML Key    | Default Value        |
|--------------|------------------|----------------------|-------------|----------------------|
| redis_url    | --redis-url      | REDIS_URL            | redis_url   | redis://127.0.0.1/   |
| host         | --host           | HOST                 | host        | 127.0.0.1            |
| port         | --port           | PORT                 | port        | 3000                 |
| log_level    | --log-level      | LOG_LEVEL            | log_level   | info                 |
| N/A (meta)   | --config         | CONFIG               | N/A         | config.yaml          |

**Note**: 모든 소스에서 동일한 이름 사용 (일관성 유지, FR-008)

---

## Error States

### 1. File Parsing Error
**Trigger**: config.yaml이 존재하지만 YAML 문법 오류  
**Behavior**: Figment가 라인 번호와 함께 오류 출력, 서버 시작 중단  
**Example**:
```
Error: 설정 파일 로딩 실패
  Caused by: invalid YAML at line 3, column 5: unexpected character
```

### 2. Type Validation Error
**Trigger**: 설정값이 타입과 맞지 않음 (예: port에 "abc" 입력)  
**Behavior**: Figment 역직렬화 실패, 서버 시작 중단  
**Example**:
```
Error: invalid type: string "abc", expected u16 for field `port`
```

### 3. Business Rule Validation Error
**Trigger**: validate() 메서드에서 비즈니스 규칙 위반  
**Behavior**: anyhow::bail! 호출, 서버 시작 중단  
**Example**:
```
Error: 포트는 1-65535 범위여야 합니다: 99999
```

### 4. Config File Not Found (Non-Error)
**Trigger**: --config로 지정된 파일이 없음 (기본 config.yaml 포함)  
**Behavior**: Figment가 해당 레이어 스킵, ENV/CLI만으로 병합 진행  
**Example**: (로그 없음, 정상 동작)

---

## Testing Scenarios

### Unit Tests (common/src/config.rs)
1. **기본값 테스트**: 모든 소스 없이 Config::default() 호출 시 기본값 사용
2. **우선순위 테스트**: 동일 필드를 YAML/ENV/CLI에 다르게 설정 후 CLI 값 확인
3. **검증 실패 테스트**: 포트 0 입력 시 validate() 실패
4. **민감 정보 마스킹**: Debug 출력 시 redis_url이 마스킹됨

### Integration Tests (tests/config_test.rs)
1. **파일 + ENV 병합**: config.yaml과 환경 변수 동시 설정 후 ENV 우선 확인
2. **상대 경로 해석**: --config ./test.yaml 입력 시 바이너리 디렉토리 기준 해석
3. **YAML 파싱 오류**: 잘못된 YAML 파일 로드 시 명확한 오류 메시지
4. **.env 로딩**: .env 파일 생성 후 환경 변수로 로드 확인

---

## Migration Plan

### 기존 코드 (main.rs)
```rust
let redis_url = std::env::var("REDIS_URL")
    .unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
```

### 새 코드 (main.rs)
```rust
dotenvy::dotenv().ok();

let config = Config::load()
    .context("설정 로딩 실패")?;

config.validate()
    .context("설정 검증 실패")?;

tracing::info!("서버 설정: {:?}", config);

let redis_client = Client::open(config.redis_url.clone())?;
// ...
```

**Breaking Changes**: 없음 (환경 변수 REDIS_URL은 그대로 지원)
