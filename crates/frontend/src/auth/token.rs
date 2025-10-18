use base64::{engine::general_purpose, Engine as _};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub exp: u64,    // 만료 시간 (Unix timestamp)
    pub iat: u64,    // 발급 시간
    pub iss: String, // 발급자
}

pub fn decode_claims(token: &str) -> Result<Claims, String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid JWT format: expected 3 parts".to_string());
    }

    let payload_b64 = parts[1];

    let payload_bytes = general_purpose::URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    let claims: Claims =
        serde_json::from_slice(&payload_bytes).map_err(|e| format!("JSON parse failed: {}", e))?;

    Ok(claims)
}
