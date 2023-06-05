use std::sync::{Arc, Mutex};

use ntex::web;

use crate::server::GameServer;

mod error;
mod event;
mod server;
mod network;
mod rooms;

#[ntex::main]
async fn main() -> std::io::Result<()> {
    let server = Arc::new(Mutex::new(GameServer::new()));

    let port = 3000;
    println!("Server listening on port: {}", port);

    web::server(move || {
        web::App::new()
            .wrap(web::middleware::Logger::default())
            .state(server.clone())
            .service(network::ws_index)
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}
