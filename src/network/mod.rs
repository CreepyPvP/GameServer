use std::{cell::RefCell, io, rc::Rc, time::Duration, time::Instant};

use futures::future::{ready, select, Either};
use ntex::service::{fn_factory_with_config, fn_shutdown, Service};
use ntex::util::Bytes;
use ntex::web;
use ntex::web::ws;
use ntex::{channel::oneshot, rt, time};
use ntex::{fn_service, pipeline};
use serde::{Serialize, Deserialize};
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);


#[derive(Serialize, Deserialize)]
struct Packet {
    channel: String,
    data: serde_json::Value,
}

struct WsSession {
    hb: Instant,
}

#[web::get("/")]
async fn ws_index(req: web::HttpRequest) -> Result<web::HttpResponse, web::Error> {
    ws::start(req, fn_factory_with_config(ws_service)).await
}

/// WebSockets service factory
async fn ws_service(
    sink: ws::WsSink,
) -> Result<impl Service<ws::Frame, Response = Option<ws::Message>, Error = io::Error>, web::Error>
{
    let state = Rc::new(RefCell::new(WsSession { hb: Instant::now() }));

    // disconnect notification
    let (tx, rx) = oneshot::channel();

    // start heartbeat task
    rt::spawn(heartbeat(state.clone(), sink, rx));

    // handler service for incoming websockets frames
    let service = fn_service(move |frame| {
        let item = match frame {
            ws::Frame::Ping(msg) => {
                state.borrow_mut().hb = Instant::now();
                Some(ws::Message::Pong(msg))
            }
            ws::Frame::Pong(_) => {
                state.borrow_mut().hb = Instant::now();
                None
            }
            ws::Frame::Text(raw) => {
                let m = String::from_utf8(Vec::from(&raw[..])).unwrap();
                let packet: Packet = serde_json::from_str(&m).unwrap();
                None
            }
            ws::Frame::Close(reason) => Some(ws::Message::Close(reason)),
            _ => None,
        };
        ready(Ok(item))
    });

    // handler service for shutdown notification that stop heartbeat task
    let on_shutdown = fn_shutdown(move || {
        let _ = tx.send(());
    });

    // pipe our service with on_shutdown callback
    Ok(pipeline(service).and_then(on_shutdown))
}

/// helper method that sends ping to client every heartbeat interval
async fn heartbeat(state: Rc<RefCell<WsSession>>, sink: ws::WsSink, mut rx: oneshot::Receiver<()>) {
    loop {
        match select(Box::pin(time::sleep(HEARTBEAT_INTERVAL)), &mut rx).await {
            Either::Left(_) => {
                // check client heartbeats
                if Instant::now().duration_since(state.borrow().hb) > CLIENT_TIMEOUT {
                    // heartbeat timed out
                    println!("Websocket Client heartbeat failed, disconnecting!");
                    return;
                }

                // send ping
                if sink
                    .send(ws::Message::Ping(Bytes::default()))
                    .await
                    .is_err()
                {
                    return;
                }
            }
            Either::Right(_) => {
                println!("Connection is dropped, stop heartbeat task");
                return;
            }
        }
    }
}
