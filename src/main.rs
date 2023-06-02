use ntex::web;
mod network;




#[ntex::main]
async fn main() -> std::io::Result<()> {
    web::server(|| {
        web::App::new()
            .wrap(web::middleware::Logger::default())
            .service(network::ws_index)
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
