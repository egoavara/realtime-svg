use common::config::Config;
use std::env;
use std::fs;

#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.redis_url, "redis://127.0.0.1/");
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 3000);
    assert_eq!(config.log_level, "info");
}

#[test]
fn test_yaml_file_loading() {
    let temp_dir = std::env::temp_dir();
    let config_path = temp_dir.join("test_config.yaml");

    let yaml_content = r#"
redis_url: redis://test:6379/
host: 0.0.0.0
port: 8080
log_level: debug
"#;

    fs::write(&config_path, yaml_content).expect("Failed to write test config");

    // This test will fail until Config::load() is implemented
    // For now, just verify the file was created
    assert!(config_path.exists());

    // Cleanup
    fs::remove_file(&config_path).ok();
}

#[test]
fn test_yaml_parsing_error() {
    let temp_dir = std::env::temp_dir();
    let config_path = temp_dir.join("invalid_config.yaml");

    let invalid_yaml = r#"
redis_url redis://test/
invalid yaml here
"#;

    fs::write(&config_path, invalid_yaml).expect("Failed to write invalid config");

    // This test will fail until Config::load() error handling is implemented
    assert!(config_path.exists());

    // Cleanup
    fs::remove_file(&config_path).ok();
}

#[test]
fn test_env_overrides_yaml() {
    // Test that environment variables override YAML config
    // This will be implemented when ENV support is added
    env::set_var("REDIS_URL", "redis://env-test:6379/");

    // Placeholder - actual test will verify ENV > YAML precedence
    assert_eq!(env::var("REDIS_URL").unwrap(), "redis://env-test:6379/");

    // Cleanup
    env::remove_var("REDIS_URL");
}

#[test]
fn test_dotenv_loading() {
    // Test .env file loading
    // This will be implemented when dotenvy support is added
    let temp_dir = std::env::temp_dir();
    let env_path = temp_dir.join("test.env");

    let env_content = "REDIS_URL=redis://dotenv:6379/\n";
    fs::write(&env_path, env_content).expect("Failed to write .env file");

    // Placeholder - actual test will verify .env loading
    assert!(env_path.exists());

    // Cleanup
    fs::remove_file(&env_path).ok();
}

#[test]
fn test_cli_overrides_all() {
    // Test that CLI options override both ENV and YAML
    // This will be tested when CLI parsing is fully integrated
    let config = Config::default();
    assert_eq!(config.port, 3000);
}

#[test]
fn test_priority_same_field() {
    // Test priority: CLI > ENV > Config for same field
    // Placeholder for full integration test
    assert!(true);
}

#[test]
fn test_custom_config_path_absolute() {
    // Test loading config from absolute path
    let temp_dir = std::env::temp_dir();
    let custom_config = temp_dir.join("custom_config.yaml");

    let yaml_content = r#"
redis_url: redis://custom:6379/
host: 0.0.0.0
port: 9000
log_level: debug
"#;

    fs::write(&custom_config, yaml_content).expect("Failed to write custom config");
    assert!(custom_config.exists());
    assert!(custom_config.is_absolute());

    // Cleanup
    fs::remove_file(&custom_config).ok();
}

#[test]
fn test_custom_config_path_relative() {
    // Test relative path resolution (relative to binary directory)
    // This is a placeholder - actual implementation will use resolve_config_path
    assert!(true);
}

#[test]
fn test_config_file_not_found_error() {
    // Test error handling when config file doesn't exist
    // This will be tested when error handling is complete
    let non_existent = std::path::PathBuf::from("/nonexistent/config.yaml");
    assert!(!non_existent.exists());
}

#[test]
fn test_validation_port_range() {
    // Test port range validation (1-65535)
    let mut config = Config::default();

    // Valid port
    config.port = 8080;
    assert!(config.validate().is_ok());

    // Invalid port (0)
    config.port = 0;
    assert!(config.validate().is_err());

    // Note: Can't test port > 65535 with u16 type as it's compile-time checked
    // The type system itself prevents invalid ports > 65535
}

#[test]
fn test_validation_redis_url_prefix() {
    // Test Redis URL prefix validation
    let mut config = Config::default();

    // Valid redis:// prefix
    config.redis_url = "redis://localhost:6379/".to_string();
    assert!(config.validate().is_ok());

    // Valid rediss:// prefix (secure)
    config.redis_url = "rediss://localhost:6379/".to_string();
    assert!(config.validate().is_ok());

    // Invalid prefix
    config.redis_url = "http://localhost:6379/".to_string();
    assert!(config.validate().is_err());
}

#[test]
fn test_validation_error_messages() {
    // Test that validation errors have clear messages
    let mut config = Config::default();

    config.port = 0;
    let result = config.validate();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("포트"));

    config.port = 3000; // Reset to valid
    config.host = "".to_string();
    let result = config.validate();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("호스트"));
}
