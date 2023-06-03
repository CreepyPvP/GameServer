use std::rc::Rc;

use ntex::web;
use rooms::Room;

mod error;
mod event;
mod network;
mod rooms;

#[ntex::main]
async fn main() -> std::io::Result<()> {
    let port = 3000;
    println!("Server listening on port: {}", port);

    web::server(|| {
        web::App::new()
            .wrap(web::middleware::Logger::default())
            .state(Rc::new(Room::WaitingRoom))
            .service(network::ws_index)
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}
