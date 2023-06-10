use redis::aio::Connection;

use crate::error::AppError;

pub async fn connect_to_redis() -> Result<Connection, AppError> {
    let client = redis::Client::open("redis://redis/")?;
    let con = client.get_async_connection().await?;
    Ok(con)
}
