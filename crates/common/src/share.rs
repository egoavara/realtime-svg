use std::sync::Arc;

use argon2::password_hash::SaltString;
use argon2::Argon2;
use jsonwebtoken::{DecodingKey, EncodingKey};
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use rsa::pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey};
use rsa::RsaPrivateKey;
use tokio::sync::OnceCell;

use crate::errors::ApiError;

/// Redis keys for storing RSA key pair in PEM format
const RSA_PRIVATE_PEM: &str = ".realtime-svg:rsa:private_pem";
const RSA_PUBLIC_PEM: &str = ".realtime-svg:rsa:public_pem";
const PASSWORD_SALT: &str = ".realtime-svg:password_salt";

/// In-memory cache for JWK (JSON Web Key) encoding/decoding keys
///
/// Keys are loaded from Redis once on first use and cached in memory
/// to avoid Redis lookups on every JWT operation.
///
/// # Thread Safety
/// Uses `Arc<OnceCell>` for thread-safe lazy initialization
#[derive(Clone, Debug)]
pub struct ShareState {
    encoding_key: Arc<OnceCell<EncodingKey>>,
    decoding_key: Arc<OnceCell<DecodingKey>>,
    salt: Arc<OnceCell<SaltString>>,
    argon2: Arc<Argon2<'static>>,
}

impl ShareState {
    pub fn new() -> Self {
        Self {
            encoding_key: Arc::new(OnceCell::new()),
            decoding_key: Arc::new(OnceCell::new()),
            salt: Arc::new(OnceCell::new()),
            argon2: Arc::new(Argon2::default()),
        }
    }

    pub fn argon2(&self) -> &Argon2<'static> {
        &self.argon2
    }

    pub async fn get_salt(&self, redis: &redis::Client) -> Result<&SaltString, ApiError> {
        self.salt
            .get_or_try_init(|| async {
                let mut conn = redis
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ApiError::RedisError(e.to_string()))?;
                let salt_str: String = conn
                    .get(PASSWORD_SALT)
                    .await
                    .map_err(|e| ApiError::RedisError(e.to_string()))?;
                let salt = SaltString::from_b64(&salt_str).map_err(|e| {
                    ApiError::InternalError(format!("Failed to create SaltString: {}", e))
                })?;
                Ok(salt)
            })
            .await
    }

    /// Gets the decoding key (RSA public key) for JWT verification
    ///
    /// Loads from Redis on first call, then caches in memory.
    /// Subsequent calls return the cached key without Redis access.
    pub async fn get_decoding_key(&self, redis: &redis::Client) -> Result<&DecodingKey, ApiError> {
        self.decoding_key
            .get_or_try_init(|| async {
                let mut conn = redis
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ApiError::RedisError(e.to_string()))?;
                let pem: String = conn
                    .get(RSA_PUBLIC_PEM)
                    .await
                    .map_err(|e| ApiError::RedisError(e.to_string()))?;
                let key = DecodingKey::from_rsa_pem(pem.as_bytes()).map_err(|e| {
                    ApiError::InternalError(format!("Failed to create DecodingKey: {}", e))
                })?;
                Ok(key)
            })
            .await
    }

    /// Gets the encoding key (RSA private key) for JWT signing
    ///
    /// Loads from Redis on first call, then caches in memory.
    /// Subsequent calls return the cached key without Redis access.
    pub async fn get_encoding_key(&self, redis: &redis::Client) -> Result<&EncodingKey, ApiError> {
        self.encoding_key
            .get_or_try_init(|| async {
                let mut conn = redis
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ApiError::RedisError(e.to_string()))?;
                let pem: String = conn
                    .get(RSA_PRIVATE_PEM)
                    .await
                    .map_err(|e| ApiError::RedisError(e.to_string()))?;
                let key = EncodingKey::from_rsa_pem(pem.as_bytes()).map_err(|e| {
                    ApiError::InternalError(format!("Failed to create EncodingKey: {}", e))
                })?;
                Ok(key)
            })
            .await
    }
}

impl Default for ShareState {
    fn default() -> Self {
        Self::new()
    }
}

/// Initializes RSA key pair in Redis if not already present
///
/// # Key Generation
/// - Algorithm: RSA-2048 bits
/// - Format: PKCS#1 PEM encoding
/// - Storage: Redis keys `rsa:private_pem` and `rsa:public_pem`
///
/// # Atomicity
/// Uses Redis `SET NX` to prevent race conditions when multiple
/// instances try to initialize keys simultaneously. Only the first
/// instance succeeds, others skip initialization.
///
/// # Idempotency
/// Safe to call multiple times - checks for existing keys first.
/// Logs info message if keys already exist.
///
/// # Usage
/// Should be called once during application startup before
/// handling any requests that require JWT authentication.
pub async fn initialize_redis(redis: &redis::Client) -> Result<(), ApiError> {
    let mut conn = redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| ApiError::RedisError(e.to_string()))?;

    initialize_jwk(&mut conn).await?;
    initialize_salt(&mut conn).await?;
    Ok(())
}

async fn initialize_jwk(conn: &mut MultiplexedConnection) -> Result<(), ApiError> {
    let exists: bool = conn
        .exists(RSA_PRIVATE_PEM)
        .await
        .map_err(|e| ApiError::RedisError(e.to_string()))?;

    if exists {
        tracing::info!("RSA keys already exist in Redis");
        return Ok(());
    }

    let rsa_key = RsaPrivateKey::new(&mut rand::thread_rng(), 2048)
        .map_err(|e| ApiError::InternalError(format!("Failed to generate RSA key: {}", e)))?;

    let private_pem = rsa_key
        .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .map_err(|e| ApiError::InternalError(format!("Failed to encode private key: {}", e)))?;

    let public_key = rsa_key.to_public_key();
    let public_pem = public_key
        .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .map_err(|e| ApiError::InternalError(format!("Failed to encode public key: {}", e)))?;

    let set: bool = conn
        .set_nx(RSA_PRIVATE_PEM, private_pem.as_str())
        .await
        .map_err(|e| ApiError::RedisError(e.to_string()))?;

    if set {
        let _: () = conn
            .set(RSA_PUBLIC_PEM, public_pem.as_str())
            .await
            .map_err(|e| ApiError::RedisError(e.to_string()))?;
        tracing::info!("Created new RSA keys in Redis");
    } else {
        tracing::info!("RSA keys were created by another instance");
    }

    Ok(())
}

async fn initialize_salt(conn: &mut MultiplexedConnection) -> Result<(), ApiError> {
    let exists: bool = conn
        .exists(PASSWORD_SALT)
        .await
        .map_err(|e| ApiError::RedisError(e.to_string()))?;

    if exists {
        tracing::info!("Password salt already exists in Redis");
        return Ok(());
    }

    let salt = SaltString::generate(&mut rand::thread_rng());
    let set: bool = conn
        .set_nx(PASSWORD_SALT, salt.as_str())
        .await
        .map_err(|e| ApiError::RedisError(e.to_string()))?;

    if set {
        tracing::info!("Created new password salt in Redis");
    } else {
        tracing::info!("Password salt was created by another instance");
    }

    Ok(())
}
