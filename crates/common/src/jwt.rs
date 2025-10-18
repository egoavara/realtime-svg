use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::errors::ApiError;

const ISSUER: &str = "realtime-svg";

/// JWT Claims structure following RFC 7519 standard
///
/// # Fields
/// - `sub` (subject): User identifier
/// - `exp` (expiration): Unix timestamp when token expires
/// - `iat` (issued at): Unix timestamp when token was issued
/// - `iss` (issuer): System identifier (always "realtime-svg")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
}

/// Creates a JWT token for a user with RSA-2048 signature
///
/// # Arguments
/// - `user_id`: User identifier to be stored in the `sub` claim
/// - `encoding_key`: RSA private key for signing (from JWK cache)
/// - `ttl_seconds`: Optional TTL in seconds (default: 24 hours)
///
/// # Returns
/// Signed JWT token string in format: `header.payload.signature`
///
/// # Algorithm
/// Uses RS256 (RSA with SHA-256) for signature generation
pub fn create_token(
    user_id: &str,
    encoding_key: &EncodingKey,
    ttl_seconds: u64,
) -> Result<String, ApiError> {
    let now = Utc::now();
    let exp = now + Duration::seconds(ttl_seconds as i64);

    let claims = Claims {
        sub: user_id.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
        iss: ISSUER.to_string(),
    };

    let header = Header::new(Algorithm::RS256);
    let token = encode(&header, &claims, encoding_key)
        .map_err(|e| ApiError::InternalError(format!("Failed to encode JWT: {}", e)))?;

    tracing::info!("Issued JWT token for user: {}", user_id);
    Ok(token)
}

/// Verifies JWT token signature and validates claims
///
/// # Arguments
/// - `token`: JWT token string to verify
/// - `decoding_key`: RSA public key for verification (from JWK cache)
///
/// # Returns
/// Decoded claims if token is valid
///
/// # Validation Rules
/// - Signature must match RSA public key (RS256 algorithm)
/// - Issuer (`iss`) must be "realtime-svg"
/// - Expiration (`exp`) must be in the future (with 60s leeway)
/// - Token structure must be valid JWT format
///
/// # Errors
/// Returns `ApiError::Unauthorized` if:
/// - Signature is invalid
/// - Token has expired
/// - Issuer doesn't match
/// - Token format is malformed
pub fn verify_token(token: &str, decoding_key: &DecodingKey) -> Result<Claims, ApiError> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[ISSUER]);

    let token_data = decode::<Claims>(token, decoding_key, &validation).map_err(|e| {
        tracing::warn!("JWT verification failed: {}", e);
        ApiError::Unauthorized(format!("Invalid JWT token: {}", e))
    })?;

    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{DecodingKey, EncodingKey};
    use rsa::pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey};
    use rsa::RsaPrivateKey;

    fn generate_test_keys() -> (EncodingKey, DecodingKey) {
        let rsa_key = RsaPrivateKey::new(&mut rand::thread_rng(), 2048).unwrap();
        let private_pem = rsa_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF).unwrap();
        let public_key = rsa_key.to_public_key();
        let public_pem = public_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF).unwrap();

        let encoding_key = EncodingKey::from_rsa_pem(private_pem.as_bytes()).unwrap();
        let decoding_key = DecodingKey::from_rsa_pem(public_pem.as_bytes()).unwrap();

        (encoding_key, decoding_key)
    }

    #[test]
    fn test_create_token() {
        let (encoding_key, _) = generate_test_keys();
        let token = create_token("test_user", &encoding_key, 3600).unwrap();

        assert!(!token.is_empty());
        assert!(token.contains('.'));
    }

    #[test]
    fn test_verify_valid_token() {
        let (encoding_key, decoding_key) = generate_test_keys();
        let token = create_token("test_user", &encoding_key, 3600).unwrap();

        let claims = verify_token(&token, &decoding_key).unwrap();
        assert_eq!(claims.sub, "test_user");
        assert_eq!(claims.iss, ISSUER);
    }

    #[test]
    fn test_verify_token_expiration() {
        let (encoding_key, decoding_key) = generate_test_keys();

        let now = chrono::Utc::now();
        let exp = now - chrono::Duration::hours(1);

        let claims = Claims {
            sub: "test_user".to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
            iss: ISSUER.to_string(),
        };

        let header = Header::new(Algorithm::RS256);
        let token = encode(&header, &claims, &encoding_key).unwrap();

        let result = verify_token(&token, &decoding_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_invalid_signature() {
        let (encoding_key1, _) = generate_test_keys();
        let (_, decoding_key2) = generate_test_keys();

        let token = create_token("test_user", &encoding_key1, 3600).unwrap();
        let result = verify_token(&token, &decoding_key2);

        assert!(result.is_err());
    }
}
