use crate::{DirectError::*, Result};
use redis::{aio::Connection, AsyncCommands, FromRedisValue};

pub async fn get_con(client: redis::Client) -> Result<Connection> {
    client
        .get_async_connection()
        .await
        .map_err(|e| RedisClientError(e).into())
}

pub async fn set_str(
    con: &mut Connection,
    key: &str,
    value: &str,
    ttl_seconds: usize,
) -> Result<()> {
    con.set(key, value).await.map_err(RedisCMDError)?;
    if ttl_seconds > 0 {
        con.expire(key, ttl_seconds).await.map_err(RedisCMDError)?;
    }
    Ok(())
}

pub async fn get_str(con: &mut Connection, key: &str) -> Result<String> {
    let value = con.get(key).await.map_err(RedisCMDError)?;
    FromRedisValue::from_redis_value(&value).map_err(|e| RedisTypeError(e).into())
}
