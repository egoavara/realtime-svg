# Research: CLI Configuration Enhancement

**Feature**: 005-cli-config-enhancement  
**Date**: 2025-10-18  
**Status**: Complete

이 문서는 clap, figment, dotenvy를 사용한 계층적 설정 시스템 구현에 필요한 기술적 결정사항을 정리한다.

## 1. Figment Configuration Merging Strategy

### Decision
Figment의 `Provider` trait를 사용하여 설정 소스를 우선순위 순서로 병합한다:
```rust
Figment::new()
    .merge(Yaml::file("config.yaml"))      // 최하위 우선순위
    .merge(Env::prefixed("APP_"))          // 중간 우선순위
    .merge(Serialized::defaults(cli_args)) // 최상위 우선순위
```

### Rationale
- Figment는 여러 설정 소스를 자동으로 병합하는 "layer-based" 아키텍처를 제공
- 나중에 merge()된 provider가 이전 값을 오버라이드 (last-write-wins)
- 타입 안전성: Figment는 serde를 사용하여 설정을 구조체로 역직렬화
- 오류 처리: 각 레이어의 파싱 오류를 세밀하게 추적 가능

### Alternatives Considered
- **config-rs**: 비슷한 기능이지만 Figment가 Rust 생태계에서 더 표준적이고 문서가 풍부함
- **수동 병합**: HashMap을 사용한 수동 병합은 타입 안전성 부족 및 코드 복잡도 증가

### References
- https://docs.rs/figment/latest/figment/
- https://github.com/SergioBenitez/Figment/tree/master/examples

---

## 2. Clap CLI Argument Parsing Integration

### Decision
Clap의 `derive` API를 사용하여 CLI 인자를 구조체로 파싱하고, `#[arg(env = "VAR_NAME")]` 속성으로 환경 변수 폴백을 지원한다:
```rust
#[derive(Parser, Serialize)]
struct CliArgs {
    #[arg(long, env = "REDIS_URL")]
    redis_url: Option<String>,
    
    #[arg(long, env = "PORT", default_value = "3000")]
    port: u16,
    
    #[arg(long, default_value = "config.yaml")]
    config: PathBuf,
}
```

### Rationale
- Derive API는 구조체 정의만으로 자동 파싱/검증/도움말 생성
- `env` 속성: 환경 변수를 자동으로 읽어 CLI 인자 fallback 제공
- Figment와의 통합: Clap 구조체를 `Serialized::defaults()로 직접 전달 가능
- 타입 안전성: 포트 번호 등을 컴파일 타임에 u16으로 검증

### Alternatives Considered
- **Builder API**: 더 유연하지만 boilerplate 코드 증가
- **structopt** (deprecated): clap 4.x derive API로 대체됨

### References
- https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html
- https://docs.rs/clap/latest/clap/struct.Arg.html#method.env

---

## 3. Dotenvy .env File Loading Order

### Decision
애플리케이션 시작 시 `dotenvy::dotenv().ok()`를 **가장 먼저** 호출하여 .env 파일을 시스템 환경 변수로 로드한다:
```rust
fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok(); // .env 파일 로드 (파일 없어도 에러 아님)
    
    let config = Config::load()?; // CLI + ENV + File 병합
    // ...
}
```

### Rationale
- `dotenv()`는 .env 파일의 변수를 프로세스 환경 변수에 추가 (기존 환경 변수는 오버라이드 **안 함**)
- 시스템 환경 변수 우선: .env와 시스템 ENV가 충돌하면 시스템 ENV가 우선됨 (dotenvy 기본 동작)
- `.ok()` 사용: .env 파일이 없어도 에러 없이 계속 진행 (선택적 기능)
- Figment Env provider와 통합: dotenv() 이후 환경 변수를 Figment가 자동으로 읽음

### Alternatives Considered
- **파일 존재 여부 체크**: `.expect()` 사용 시 .env가 필수가 되어 배포 유연성 감소
- **다른 .env 라이브러리**: dotenv (archived), envy (기능 부족) → dotenvy가 현재 유지보수되는 표준

### References
- https://docs.rs/dotenvy/latest/dotenvy/
- https://12factor.net/config (환경 변수 우선순위 철학)

---

## 4. Binary Executable Path Resolution

### Decision
`std::env::current_exe()`를 사용하여 바이너리 실행 경로를 찾고, `--config` 옵션이 상대 경로일 경우 바이너리 디렉토리 기준으로 해석한다:
```rust
fn resolve_config_path(config_arg: &Path) -> anyhow::Result<PathBuf> {
    if config_arg.is_absolute() {
        return Ok(config_arg.to_path_buf());
    }
    
    let exe_dir = std::env::current_exe()?
        .parent()
        .context("실행 파일 경로를 찾을 수 없습니다")?
        .to_path_buf();
    
    Ok(exe_dir.join(config_arg))
}
```

### Rationale
- `current_exe()`: 실행 파일의 절대 경로 반환 (심볼릭 링크 해석됨)
- 바이너리 디렉토리 기준: 배포 환경에서 예측 가능한 동작 (작업 디렉토리와 무관)
- 절대 경로 지원: `/etc/app/config.yaml` 같은 시스템 경로도 허용
- 오류 처리: 파일이 없으면 명확한 에러 메시지와 함께 실패

### Alternatives Considered
- **`std::env::current_dir()`**: 작업 디렉토리는 호출 위치에 따라 달라져 배포 환경에서 불안정
- **환경 변수로 경로 지정**: 추가 복잡도이며, --config 옵션으로 충분

### References
- https://doc.rust-lang.org/std/env/fn.current_exe.html
- https://doc.rust-lang.org/std/path/struct.Path.html#method.is_absolute

---

## 5. Configuration Validation Strategy

### Decision
설정 구조체에 사용자 정의 `validate()` 메서드를 구현하여 비즈니스 규칙 검증:
```rust
impl Config {
    pub fn validate(&self) -> anyhow::Result<()> {
        // 포트 범위 검증
        if self.port == 0 || self.port > 65535 {
            anyhow::bail!("포트는 1-65535 범위여야 합니다: {}", self.port);
        }
        
        // Redis URL 형식 검증
        if !self.redis_url.starts_with("redis://") && !self.redis_url.starts_with("rediss://") {
            anyhow::bail!("Redis URL은 redis:// 또는 rediss:// 로 시작해야 합니다");
        }
        
        Ok(())
    }
}
```

### Rationale
- **레이어드 검증**: serde는 타입 검증 (예: 문자열을 u16으로 파싱), validate()는 비즈니스 규칙 검증
- **명확한 오류 메시지**: anyhow::bail!로 사용자 친화적인 에러 제공
- **서버 시작 전 검증**: main()에서 `config.validate()?` 호출하여 잘못된 설정으로 서버가 시작되지 않도록 보장

### Alternatives Considered
- **validator crate**: derive 기반 검증이지만 커스텀 규칙에 대해 과도하게 복잡함
- **검증 없음**: 런타임 에러 발생 시 디버깅 어려움 (예: 포트 0으로 바인딩 실패)

### References
- https://docs.rs/anyhow/latest/anyhow/macro.bail.html
- https://docs.serde.rs/serde/trait.Deserialize.html (기본 타입 검증)

---

## 6. Sensitive Information Masking in Logs

### Decision
설정 로그 출력 시 민감 정보 필드를 `***REDACTED***`로 마스킹하는 커스텀 Debug trait 구현:
```rust
impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Config")
            .field("redis_url", &"***REDACTED***")
            .field("port", &self.port)
            .field("host", &self.host)
            .finish()
    }
}
```

### Rationale
- **보안**: Redis URL에 비밀번호가 포함될 수 있음 (redis://:password@host/)
- **로그 안전성**: `tracing::info!("{:?}", config)` 호출 시 자동 마스킹
- **선택적 마스킹**: 민감하지 않은 필드(port, host)는 그대로 출력하여 디버깅 용이성 유지

### Alternatives Considered
- **serde 직렬화 스킵**: `#[serde(skip_serializing)]`는 설정 로딩에 영향을 줌
- **수동 마스킹**: 로그 호출마다 수동으로 마스킹하면 누락 위험

### References
- https://doc.rust-lang.org/std/fmt/trait.Debug.html
- https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html

---

## 7. YAML File Parsing Error Handling

### Decision
Figment의 `Yaml::file()` provider를 사용하고, 파일이 없을 경우 무시하되 파싱 오류는 명확히 보고:
```rust
Figment::new()
    .merge(Yaml::file("config.yaml").nested()) // 파일 없으면 스킵, 있으면 파싱
    .extract::<Config>()
    .context("설정 파일 로딩 실패")?
```

### Rationale
- **선택적 파일**: 설정 파일이 없어도 ENV/CLI로 충분히 동작 (기본값 사용)
- **파싱 오류 명확화**: YAML 문법 오류 시 Figment가 라인 번호와 함께 오류 출력
- `context()`: anyhow로 오류 체인을 추가하여 디버깅 정보 제공

### Alternatives Considered
- **파일 필수화**: `.required()` 사용 시 배포 유연성 감소
- **JSON 지원**: 현재는 YAML만 지원 (spec의 "Out of Scope" 명시)

### References
- https://docs.rs/figment/latest/figment/providers/struct.Yaml.html
- https://yaml.org/spec/1.2/spec.html (YAML 1.2 spec)

---

## 8. Best Practices for Rust CLI Configuration

### Decision
다음 12-factor app 원칙을 따른다:
1. **환경 변수 우선**: 컨테이너 환경 호환성
2. **설정 파일 선택적**: 로컬 개발 편의성
3. **CLI 최우선**: 일시적 오버라이드
4. **기본값 제공**: 합리적인 fallback (redis://127.0.0.1/)
5. **검증 즉시**: 서버 시작 전 설정 검증

### Rationale
- **12-factor app**: 클라우드 네이티브 배포 표준
- **Kubernetes 호환**: ConfigMap (파일) + Secret (ENV) 패턴 지원
- **개발자 경험**: 로컬에서 .env 사용, 프로덕션에서 K8s Secret 사용 가능

### References
- https://12factor.net/config
- https://kubernetes.io/docs/concepts/configuration/configmap/

---

## Implementation Checklist

- [ ] common/src/config.rs: Config 구조체 및 load() 함수
- [ ] common/src/config.rs: validate() 메서드
- [ ] common/src/config.rs: 민감 정보 마스킹 Debug impl
- [ ] backend/src/main.rs: dotenvy::dotenv() 호출
- [ ] backend/src/main.rs: Config::load() 호출 및 검증
- [ ] tests/config_test.rs: 우선순위 병합 테스트
- [ ] tests/config_test.rs: 파일 파싱 오류 테스트
- [ ] config.yaml.example: 문서용 예제 파일
- [ ] .env.example: 문서용 예제 파일
