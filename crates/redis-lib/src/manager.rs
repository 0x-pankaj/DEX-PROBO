use std::sync::Arc;

use redis::{
    AsyncCommands, Client,
    streams::{StreamReadOptions, StreamReadReply},
};
use serde::{Serialize, de::DeserializeOwned};

#[derive(Debug, Clone)]
pub struct RedisStore {
    client: Arc<Client>,
}

type BoxError = Box<dyn std::error::Error + Send + Sync>;

impl RedisStore {
    pub async fn new(redis_url: &str) -> Result<Self, BoxError> {
        let client = Client::open(redis_url)?;
        Ok(RedisStore {
            client: Arc::new(client),
        })
    }

    pub async fn add_message_to_stream<T: Serialize>(
        &self,
        message: &T,
    ) -> Result<String, BoxError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let serialized = serde_json::to_string(message)?;
        let stream_id: String = conn
            .xadd("order_stream", "*", &[("data", serialized)])
            .await?;
        Ok(stream_id)
    }

    pub async fn get_message_from_stream<T: DeserializeOwned>(
        &self,
        consumer_group: &str,
        consumer_name: &str,
    ) -> Result<Vec<(String, T)>, BoxError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        //creating consumer group if not present
        let _: Result<(), redis::RedisError> = conn
            .xgroup_create_mkstream("order_stream", consumer_group, "0")
            .await;

        let opts = StreamReadOptions::default()
            .group(consumer_group, consumer_name)
            .count(1)
            .block(1);

        let result: StreamReadReply = conn.xread_options(&["order_stream"], &[">"], &opts).await?;

        let mut entries = Vec::new();
        for key in result.keys {
            for entry_id in key.ids {
                let data: String = entry_id
                    .get("data")
                    .ok_or("Missing data filed in stream entry")?;
                let message: T = serde_json::from_str(&data)?;
                entries.push((entry_id.id, message));
            }
        }
        Ok(entries)
    }

    pub async fn ack_stream(
        &self,
        stream_name: &str,
        consumer_group: &str,
        id: &str,
    ) -> Result<i32, redis::RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let ack_count: i32 = conn.xack(stream_name, consumer_group, &[id]).await?;
        Ok(ack_count)
    }
}
