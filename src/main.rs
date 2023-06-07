use error::GameError;
use ntex::web;
use redis::RedisResult;

use crate::server::GameServer;

mod error;
mod event;
mod network;
mod rooms;
mod server;


fn redis_connection() -> RedisResult<()> {
    println!("Connecting to redis");
    let client = redis::Client::open("redis://redis/")?;
    let mut con = client.get_connection()?;
    println!("Connected to redis");
    Ok(())
}

#[ntex::main]
async fn main() -> Result<(), GameError> {
    redis_connection()?;

    let srv = GameServer::start();
    let port = 3000;
    println!("Server listening on port: {}", port);

    web::server(move || {
        web::App::new()
            .wrap(web::middleware::Logger::default())
            .state(srv.clone())
            .service(network::ws_index)
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await?;

    Ok(())
}
