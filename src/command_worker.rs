use std::collections::HashMap;

use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    SinkExt, StreamExt,
};
use ntex::rt;
use redis::{aio::Connection, AsyncCommands};
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    event::client_message::ClientEvent,
    util::{redis::{connect_to_redis, connect_to_redis_sync}, stream::{merge_receiver, EventType}},
};

pub enum CmdWorkerMsg {
    Register(usize, UnboundedSender<ClientEvent>),
    Remove(usize),
}

pub struct CommandWorker {
    con: Connection,
    id: String,
    clients: HashMap<usize, UnboundedSender<ClientEvent>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum Command {
    Send(usize, String),
}

impl CommandWorker {
    async fn process(&self, cmd: Command) -> Result<(), AppError> {
        match cmd {
            Command::Send(user, msg) => {
                if let Some(mut client) = self.clients.get(&user) {
                    client.send(ClientEvent::RawMessage(msg)).await?;
                }
            }
        }

        Ok(())
    }

    async fn register_client(
        &mut self,
        id: usize,
        client: UnboundedSender<ClientEvent>,
    ) -> Result<(), AppError> {
        self.clients.insert(id, client);
        self.con.set(format!("clients:{}", id), &self.id).await?;

        Ok(())
    }

    async fn remove_client(&mut self, id: usize) -> Result<(), AppError> {
        self.clients.remove(&id);
        self.con.del(format!("clients:{}", id)).await?;

        Ok(())
    }

    async fn redis_messages(
        channel_id: String,
        mut tx: UnboundedSender<Command>,
    ) -> Result<(), AppError> {
        let mut subcon = connect_to_redis_sync().await?;
        let mut subscriber = subcon.as_pubsub();

        subscriber.subscribe(channel_id)?;
        loop {
            let msg = subscriber.get_message()?;
            let cmd: Command = match serde_json::from_str(msg.get_payload::<String>()?.as_str()) {
                Ok(res) => res,
                Err(err) => {
                    println!("Error in redis worker: {}", err);
                    continue;
                }
            };
            match tx.send(cmd).await {
                Ok(_) => (),
                Err(_) => {
                    println!("Exiting redis worker");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn start(&mut self, mut rx: UnboundedReceiver<CmdWorkerMsg>) -> Result<(), AppError> {
        let channel_id = format!("workers:{}", self.id);

        let (tredis, mut rredis) = mpsc::unbounded();
        rt::Arbiter::new().exec_fn(move || {
            rt::spawn(CommandWorker::redis_messages(channel_id, tredis));
        });

        let mut events = merge_receiver(&mut rx, &mut rredis);

        while let Some(ev) = events.next().await {
            match ev {
                EventType::A(CmdWorkerMsg::Register(id, client)) => {
                    self.register_client(id, client).await?
                }
                EventType::A(CmdWorkerMsg::Remove(id)) => self.remove_client(id).await?,
                EventType::B(cmd) => self.process(cmd).await?,
            }
        }

        println!("Exiting command thread");
        Ok(())
    }

    pub async fn create() -> Result<UnboundedSender<CmdWorkerMsg>, AppError> {
        let (tx, rx) = mpsc::unbounded::<CmdWorkerMsg>();

        let mut worker = CommandWorker {
            con: connect_to_redis().await?,
            id: uuid::Uuid::new_v4().to_string(),
            clients: HashMap::new(),
        };

        rt::Arbiter::new().exec_fn(move || {
            rt::spawn(async move {
                if let Err(err) = worker.start(rx).await {
                    println!("Error in cmd worker: {}", err);
                }
                rt::Arbiter::current().stop();
            });
        });

        Ok(tx)
    }
}
