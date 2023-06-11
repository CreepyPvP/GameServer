use error::AppError;
use ntex::web;

use crate::{command_worker::CommandWorker, server::GameServer};

mod command_worker;
mod error;
mod event;
mod network;
mod rooms;
mod server;
mod util;

#[ntex::main]
async fn main() -> Result<(), AppError> {
    let srv = GameServer::start();
    let message_worker = CommandWorker::create()
        .await
        .expect("Failed to create command worker");

    let port = 3000;
    println!("Server listening on port: {}", port);

    web::server(move || {
        web::App::new()
            .wrap(web::middleware::Logger::default())
            .state((srv.clone(), message_worker.clone()))
            .service(network::ws_index)
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await?;

    Ok(())
}
