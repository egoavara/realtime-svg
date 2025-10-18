use web_sys::{window, Storage};

const TOKEN_KEY: &str = "jwt_token";

/// 토큰 저장소 추상화
pub trait TokenStorage {
    fn get_token(&self) -> Option<String>;
    fn set_token(&self, token: &str) -> Result<(), String>;
    fn remove_token(&self) -> Result<(), String>;
}

/// localStorage 기반 구현
pub struct LocalTokenStorage;

impl LocalTokenStorage {
    pub fn new() -> Self {
        Self
    }

    /// localStorage 인스턴스 가져오기
    fn get_storage(&self) -> Option<Storage> {
        window()?.local_storage().ok()?
    }
}

impl TokenStorage for LocalTokenStorage {
    fn get_token(&self) -> Option<String> {
        let storage = self.get_storage()?;
        storage.get_item(TOKEN_KEY).ok()?
    }

    fn set_token(&self, token: &str) -> Result<(), String> {
        let storage = self.get_storage().ok_or("localStorage not available")?;

        storage.set_item(TOKEN_KEY, token).map_err(|e| {
            let error_msg = format!("{:?}", e);
            if error_msg.contains("QuotaExceededError") {
                "Storage quota exceeded".to_string()
            } else {
                format!("Failed to set token: {}", error_msg)
            }
        })
    }

    fn remove_token(&self) -> Result<(), String> {
        let storage = self.get_storage().ok_or("localStorage not available")?;

        storage
            .remove_item(TOKEN_KEY)
            .map_err(|e| format!("Failed to remove token: {:?}", e))
    }
}
