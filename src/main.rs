use ntex::web;

use crate::server::GameServer;

mod error;
mod event;
mod network;
mod rooms;
mod server;

#[ntex::main]
async fn main() -> std::io::Result<()> {
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
    .await
}
