use redis::{AsyncTypedCommands, Client};
use serde::Serialize;

use crate::{errors::ApiError, jwk::JwkCache, session_data::SessionData};

#[derive(Clone, Debug)]
pub struct AppState {
    redis_client: Client,
    jwk_cache: std::sync::Arc<JwkCache>,
}

impl AppState {
    pub fn new(redis_client: Client) -> Self {
        Self {
            redis_client: redis_client.clone(),
            jwk_cache: std::sync::Arc::new(JwkCache::new()),
        }
    }

    pub fn redis_client(&self) -> &Client {
        &self.redis_client
    }

    pub fn jwk_cache(&self) -> &JwkCache {
        &self.jwk_cache
    }

    pub async fn connection_redis(&self) -> Result<redis::aio::MultiplexedConnection, ApiError> {
        let conn = self.redis_client.get_multiplexed_async_connection().await?;
        Ok(conn)
    }

    pub async fn publish<S: Serialize>(&self, channel: &str, message: &S) -> Result<(), ApiError> {
        let json = serde_json::to_string(message)?;
        self.redis_client
            .get_multiplexed_async_connection()
            .await?
            .publish(channel, json)
            .await?;
        Ok(())
    }

    pub async fn pubsub(&self) -> Result<redis::aio::PubSub, ApiError> {
        let pubsub = self.redis_client.get_async_pubsub().await?;
        Ok(pubsub)
    }

    pub async fn set_session(
        &self,
        session_id: &str,
        session: &SessionData,
        ttl_seconds: u64,
    ) -> Result<(), ApiError> {
        let frame = session.current_frame();
        let session = serde_json::to_string(session)?;
        let frame = serde_json::to_string(&frame)?;
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        conn.set_ex(session_id, session.as_str(), ttl_seconds)
            .await?;
        conn.publish(session_id, &frame).await?;
        Ok(())
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<SessionData>, ApiError> {
        let data: Option<String> = self
            .redis_client
            .get_multiplexed_async_connection()
            .await?
            .get(session_id)
            .await?;
        match data {
            Some(json) => {
                let session = serde_json::from_str::<SessionData>(&json)?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    pub async fn set_user_session(
        &self,
        user_id: &str,
        session_id: &str,
        session: &SessionData,
        ttl_seconds: u64,
    ) -> Result<(), ApiError> {
        let key = format!("user:{}:session:{}", user_id, session_id);
        let channel = format!("user:{}:session:{}", user_id, session_id);

        let frame = session.current_frame();
        let session_json = serde_json::to_string(session)?;
        let frame_json = serde_json::to_string(&frame)?;

        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        conn.set_ex(&key, session_json.as_str(), ttl_seconds)
            .await?;
        conn.publish(&channel, &frame_json).await?;

        Ok(())
    }

    pub async fn get_user_session(
        &self,
        user_id: &str,
        session_id: &str,
    ) -> Result<Option<SessionData>, ApiError> {
        let key = format!("user:{}:session:{}", user_id, session_id);
        let data: Option<String> = self
            .redis_client
            .get_multiplexed_async_connection()
            .await?
            .get(&key)
            .await?;
        match data {
            Some(json) => {
                let session = serde_json::from_str::<SessionData>(&json)?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    pub async fn list_user_sessions(&self, user_id: &str) -> Result<Vec<String>, ApiError> {
        let pattern = format!("user:{}:session:*", user_id);
        let mut conn = self.connection_redis().await?;
        let mut cursor = 0u64;
        let mut sessions = Vec::new();

        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await
                .map_err(|e| ApiError::RedisError(e.to_string()))?;

            for key in keys {
                let prefix = format!("user:{}:session:", user_id);
                if let Some(session_id) = key.strip_prefix(&prefix) {
                    sessions.push(session_id.to_string());
                }
            }

            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }

        Ok(sessions)
    }
}
