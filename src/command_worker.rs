use std::collections::HashMap;

use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    StreamExt, io::ReadToEnd, SinkExt,
};
use ntex::{
    rt, util,
    util::Either::{Left, Right},
};
use redis::{aio::Connection, AsyncCommands};
use serde::Deserialize;

use crate::{error::AppError, event::client_message::ClientEvent, util::redis::connect_to_redis};

pub enum CmdWorkerMsg {
    Register(usize, UnboundedSender<ClientEvent>),
    Remove(usize),
}

pub struct CommandWorker {
    con: Connection,
    id: String,
    clients: HashMap<usize, UnboundedSender<ClientEvent>>,
}

#[derive(Deserialize)]
pub enum Command {
    Send(usize, String)
}

impl CommandWorker {

    async fn process(&self, data: &str) -> Result<(), AppError> {
        let cmd: Command = serde_json::from_str(data)?;
        match cmd {
            Command::Send(user, msg) => {
                if let Some(mut client) = self.clients.get(&user) {
                    client.send(ClientEvent::RawMessage(msg)).await?;
                }
            }
        }

        Ok(())
    }

    async fn register_client(&mut self, id: usize, client: UnboundedSender<ClientEvent>) -> Result<(), AppError> {
        self.clients.insert(id, client);
        self.con.set(format!("clients:{}", id), &self.id).await?;

        Ok(())
    }

    async fn remove_client(&mut self, id: usize) -> Result<(), AppError> {
        self.clients.remove(&id);
        self.con.del(format!("clients:{}", id)).await?;

        Ok(())
    }

    async fn start(&mut self, mut rx: UnboundedReceiver<CmdWorkerMsg>) -> Result<(), AppError> {
        let key = format!("workers:{}", self.id);
        loop {
            match util::select(
                rx.next(),
                self.con.brpop::<&str, Vec<String>>(key.as_str(), 1000000),
            )
            .await
            {
                Left(Some(CmdWorkerMsg::Register(id, client))) => self.register_client(id, client).await?,
                Left(Some(CmdWorkerMsg::Remove(id))) => self.remove_client(id).await?,
                Left(None) => break,
                Right(Ok(val)) => self.process(&val[1]).await?,
                Right(Err(_)) => (),
            }
        }

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
                let _ = worker.start(rx).await;
                rt::Arbiter::current().stop();
            });
        });

        Ok(tx)
    }
}
