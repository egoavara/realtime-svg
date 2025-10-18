use argon2::{PasswordHash, PasswordVerifier};
use redis::{AsyncTypedCommands, Client};
use serde::Serialize;

use crate::{errors::ApiError, session_data::SessionData, share::ShareState, user_data::UserData};

#[derive(Clone, Debug)]
pub struct AppState {
    redis_client: Client,
    share: ShareState,
}

impl AppState {
    pub fn new(redis_client: Client) -> Self {
        Self {
            redis_client: redis_client.clone(),
            share: ShareState::new(),
        }
    }

    pub fn redis_client(&self) -> &Client {
        &self.redis_client
    }

    pub fn share(&self) -> &ShareState {
        &self.share
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

    pub async fn create_user_data(
        &self,
        user_id: impl AsRef<str>,
        password: impl Into<String>,
        ttl_seconds: u64,
    ) -> Result<UserData, ApiError> {
        let share = self.share();
        let salt = share.get_salt(self.redis_client()).await?;
        let user_data = UserData::create(self.share.argon2(), salt, password.into())?;
        let user_key = format!("user:{}:data", user_id.as_ref());
        let user_json = serde_json::to_string(&user_data)?;
        let mut conn = self.connection_redis().await?;

        redis::cmd("SET")
            .arg(user_key.as_str())
            .arg(user_json.as_str())
            .arg("NX")
            .arg("EX")
            .arg(ttl_seconds)
            .query_async::<()>(&mut conn)
            .await?;
        self.get_user_data(user_id.as_ref())
            .await?
            .ok_or_else(|| ApiError::Unexpected("Failed to create user data".to_string()))
    }

    pub async fn verify_user_password(
        &self,
        user_data: &UserData,
        password: impl AsRef<str>,
    ) -> Result<bool, ApiError> {
        let share = self.share();
        let password_hash = PasswordHash::new(&user_data.password_argon2)?;
        let provided_password = password.as_ref().as_bytes();
        match share
            .argon2()
            .verify_password(provided_password, &password_hash)
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub async fn get_user_data(
        &self,
        user_id: impl AsRef<str>,
    ) -> Result<Option<UserData>, ApiError> {
        let user_key = format!("user:{}:data", user_id.as_ref());
        let data: Option<String> = self
            .redis_client
            .get_multiplexed_async_connection()
            .await?
            .get(&user_key)
            .await?;
        match data {
            Some(json) => {
                let user_data = serde_json::from_str::<UserData>(&json)?;
                Ok(Some(user_data))
            }
            None => Ok(None),
        }
    }
}
