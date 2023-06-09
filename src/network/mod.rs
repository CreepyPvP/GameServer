use std::{cell::RefCell, io, rc::Rc, time::Duration, time::Instant};

use futures::channel::mpsc::{self, UnboundedSender};
use futures::future::ready;
use futures::{SinkExt, StreamExt};
use ntex::service::{fn_factory_with_config, fn_shutdown, map_config, Service};
use ntex::util::{self, ByteString, Bytes};
use ntex::web;
use ntex::web::ws;
use ntex::{channel::oneshot, rt, time};
use ntex::{fn_service, pipeline};
use serde::{Deserialize, Serialize};

use crate::command_worker::CmdWorkerMsg;
use crate::error::AppError;
use crate::event::client_message::ClientEvent;
use crate::event::server_message::{ServerEvent, ServerMessage};
use crate::server::UserManager;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Serialize, Deserialize)]
struct Packet {
    channel: String,
    data: serde_json::Value,
}

struct WsSession {
    id: usize,
    hb: Instant,

    server: UnboundedSender<ServerEvent>,
    command_worker: UnboundedSender<CmdWorkerMsg>,
}

impl Drop for WsSession {
    fn drop(&mut self) {
        let _ = self.command_worker.send(CmdWorkerMsg::Remove(self.id));
    }
}

#[derive(Deserialize)]
struct WsRequestQuery {
    token: Option<String>,
}

#[web::get("/")]
async fn ws_index(
    web::types::Query(qs): web::types::Query<WsRequestQuery>,
    req: web::HttpRequest,
    state: web::types::State<(UnboundedSender<ServerEvent>, UnboundedSender<CmdWorkerMsg>)>,
) -> Result<web::HttpResponse, web::Error> {
    let token = qs.token;
    ws::start(
        req,
        map_config(fn_factory_with_config(ws_service), move |cfg| {
            (
                cfg,
                state.get_ref().0.clone(),
                state.get_ref().1.clone(),
                token.clone(),
            )
        }),
    )
    .await
}

async fn is_valid_token(token: &str, user_mgr: &UserManager) -> bool {
    user_mgr.token_exists(token)
}

async fn ws_service(
    (sink, mut srv, mut command_worker, token): (
        ws::WsSink,
        UnboundedSender<ServerEvent>,
        UnboundedSender<CmdWorkerMsg>,
        Option<String>,
    ),
) -> Result<impl Service<ws::Frame, Response = Option<ws::Message>, Error = io::Error>, AppError> {
    let (tx, mut rx) = mpsc::unbounded();
    srv.send(ServerEvent::Connect(tx.clone(), token)).await?;

    let id = if let Some(ClientEvent::Id(id)) = rx.next().await {
        id
    } else {
        panic!();
    };

    command_worker.send(CmdWorkerMsg::Register(id, tx)).await?;

    let state = Rc::new(RefCell::new(WsSession {
        hb: Instant::now(),
        id,
        server: srv.clone(),
        command_worker: command_worker.clone(),
    }));

    rt::spawn(messages(sink.clone(), rx));

    let (tx, rx) = oneshot::channel();
    rt::spawn(heartbeat(state.clone(), sink.clone(), srv.clone(), rx));

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
                let msg = String::from_utf8(Vec::from(&raw[..])).unwrap();
                if let Some(_) = ServerMessage::parse(msg) {}

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
        // message_worker.send(CmdWorkerMsg::Remove(id)).await;
    });

    // pipe our service with on_shutdown callback
    Ok(pipeline(service).and_then(on_shutdown))
}

async fn messages(sink: ws::WsSink, mut server: mpsc::UnboundedReceiver<ClientEvent>) {
    while let Some(msg) = server.next().await {
        match msg {
            ClientEvent::Id(_) => (),
            ClientEvent::Message(packet) => {
                let raw = packet.stringfy();
                if let Ok(raw) = raw {
                    let _ = sink.send(ws::Message::Text(ByteString::from(raw))).await;
                }
            }
            ClientEvent::RawMessage(raw) => {
                let _ = sink.send(ws::Message::Text(ByteString::from(raw))).await;
            }
        };
    }
}

async fn heartbeat(
    state: Rc<RefCell<WsSession>>,
    sink: ws::WsSink,
    mut server: mpsc::UnboundedSender<ServerEvent>,
    mut rx: oneshot::Receiver<()>,
) {
    loop {
        match util::select(Box::pin(time::sleep(HEARTBEAT_INTERVAL)), &mut rx).await {
            util::Either::Left(_) => {
                if Instant::now().duration_since(state.borrow().hb) > CLIENT_TIMEOUT {
                    println!("Websocket Client heartbeat failed, disconnecting!");
                    let _ = server.send(ServerEvent::Disconnect(state.borrow().id));
                    sink.io().close();
                    return;
                } else {
                    // send ping
                    if sink.send(ws::Message::Ping(Bytes::new())).await.is_err() {
                        return;
                    }
                }
            }
            util::Either::Right(_) => {
                println!("Connection is dropped, stop heartbeat task");
                return;
            }
        }
    }
}
