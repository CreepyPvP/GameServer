use error::AppError;
use ntex::web;

use crate::{server::GameServer, command_worker::CommandWorker};

mod error;
mod event;
mod util;
mod command_worker;
mod network;
mod rooms;
mod server;



#[ntex::main]
async fn main() -> Result<(), AppError> {
    let srv = GameServer::start();
    let port = 3000;
    println!("Server listening on port: {}", port);

    web::server(move || {
        let message_worker = CommandWorker::create();

        web::App::new()
            .wrap(web::middleware::Logger::default())
            .state((srv.clone(), message_worker))
            .service(network::ws_index)
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await?;

    Ok(())
}
