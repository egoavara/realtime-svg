use axum::{extract::State, Json};
use common::{errors::ApiError, state::AppState};
use jsonwebtoken::jwk::{
    AlgorithmParameters, CommonParameters, Jwk, JwkSet, KeyAlgorithm, RSAKeyParameters,
};

pub async fn handler(State(state): State<AppState>) -> Result<Json<JwkSet>, ApiError> {
    let mut conn = state.connection_redis().await?;

    let pem: String = redis::AsyncCommands::get(&mut conn, "rsa:public_pem")
        .await
        .map_err(|e| ApiError::RedisError(e.to_string()))?;

    use rsa::pkcs1::DecodeRsaPublicKey;

    let public_key = rsa::RsaPublicKey::from_pkcs1_pem(&pem)
        .map_err(|e| ApiError::InternalError(format!("Failed to parse public key: {}", e)))?;

    use base64::Engine;
    use rsa::traits::PublicKeyParts;

    let n = public_key.n().to_bytes_be();
    let e = public_key.e().to_bytes_be();

    let n_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&n);
    let e_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&e);

    let jwk = Jwk {
        common: CommonParameters {
            public_key_use: Some(jsonwebtoken::jwk::PublicKeyUse::Signature),
            key_algorithm: Some(KeyAlgorithm::RS256),
            key_operations: None,
            key_id: None,
            x509_url: None,
            x509_chain: None,
            x509_sha1_fingerprint: None,
            x509_sha256_fingerprint: None,
        },
        algorithm: AlgorithmParameters::RSA(RSAKeyParameters {
            key_type: jsonwebtoken::jwk::RSAKeyType::RSA,
            n: n_b64,
            e: e_b64,
        }),
    };

    let jwk_set = JwkSet { keys: vec![jwk] };

    tracing::info!("JWKS endpoint accessed");

    Ok(Json(jwk_set))
}
