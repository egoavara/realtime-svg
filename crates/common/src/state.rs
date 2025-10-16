use redis::{AsyncTypedCommands, Client};
use serde::Serialize;

use crate::{errors::ApiError, session_data::SessionData};

#[derive(Clone, Debug)]
pub struct AppState {
    redis_client: Client,
}

impl AppState {
    pub fn new(redis_client: Client) -> Self {
        Self { redis_client }
    }

    pub fn redis_client(&self) -> &Client {
        &self.redis_client
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
}
