use crate::error::AppError;

pub async fn connect_to_redis() -> Result<redis::aio::Connection, AppError> {
    let client = redis::Client::open("redis://redis/")?;
    let con = client.get_async_connection().await?;
    Ok(con)
}

pub async fn connect_to_redis_sync() -> Result<redis::Connection, AppError> {
    let client = redis::Client::open("redis://redis/")?;
    let con = client.get_connection()?;
    Ok(con)
}
