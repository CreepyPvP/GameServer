use std::sync::{Arc, Mutex};
use std::{cell::RefCell, io, rc::Rc, time::Duration, time::Instant};

use futures::future::{ready, select, Either};
use ntex::service::{fn_factory_with_config, fn_shutdown, map_config, Service};
use ntex::util::{Bytes, ByteString};
use ntex::web;
use ntex::web::ws;
use ntex::{channel::oneshot, rt, time};
use ntex::{fn_service, pipeline};
use serde::{Deserialize, Serialize};

use crate::event::{Event, RawPacket};
use crate::rooms::Room;
use crate::server::GameServer;

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

#[derive(Deserialize)]
struct WsRequestQuery {
    token: Option<String>,
}

#[web::get("/")]
async fn ws_index(
    web::types::Query(qs): web::types::Query<WsRequestQuery>,
    req: web::HttpRequest,
    server: web::types::State<Arc<Mutex<GameServer>>>,
) -> Result<web::HttpResponse, web::Error> {
    let token = qs.token;
    ws::start(
        req,
        map_config(fn_factory_with_config(ws_service), move |cfg| {
            (cfg, server.get_ref().clone(), token.clone())
        }),
    )
    .await
}

async fn is_valid_token(token: &str) -> bool {
   true 
}

async fn generate_token() -> String {
    let token = uuid::Uuid::new_v4().to_string();
    // TODO: validate token
    token
}

/// WebSockets service factory
async fn ws_service(
    (sink, server, token): (ws::WsSink, Arc<Mutex<GameServer>>, Option<String>),
) -> Result<impl Service<ws::Frame, Response = Option<ws::Message>, Error = io::Error>, web::Error>
{
    let state = Rc::new(RefCell::new(WsSession {
        hb: Instant::now(),
    }));

    // disconnect notification
    let (tx, rx) = oneshot::channel();

    // start heartbeat task
    rt::spawn(heartbeat(state.clone(), sink.clone(), rx));

    // authentication
    let token = match token {
        Some(token) if is_valid_token(&token).await => token,
        _ => {
            generate_token().await
        }
    };

    println!("Got client token: {}", token);
    let user_id = server.lock().unwrap().get_or_create_user(token.clone());
    println!("Got user id: {}", user_id);
    
    let auth_packet = Event::SetAuthToken{token};
    sink.send(ws::Message::Text(ByteString::from(auth_packet.stringfy().unwrap()))).await;

    // state.borrow().room.on(Event::Connect);

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
                let packet: RawPacket = serde_json::from_str(&m).unwrap();
                let event = Event::parse(packet);

                match event {
                    Ok(event) => {}, // state.borrow().room.on(event),
                    Err(err) => println!("{}", err),
                }

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
