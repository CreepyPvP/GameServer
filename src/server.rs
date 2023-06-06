use crate::{rooms::Room, event::server_message::ServerEvent};
use crate::event::client_message::ClientEvent;
use futures::SinkExt;
use futures::{
    channel::mpsc::{self, UnboundedSender},
    StreamExt,
};
use ntex::rt;
use std::{collections::HashMap, rc::Rc};

pub struct User {
    room: Rc<Room>,
    client: Option<UnboundedSender<ClientEvent>>,
}

pub struct UserManager {
    id_counter: usize,
    sessions: HashMap<usize, User>,
    token_store: HashMap<String, usize>,
}

impl<'a> UserManager {
    fn new() -> Self {
        UserManager {
            id_counter: 0,
            sessions: HashMap::new(),
            token_store: HashMap::new(),
        }
    }

    fn generate_token(&self) -> String {
        loop {
            let token = uuid::Uuid::new_v4().to_string();
            if !self.token_exists(&token) {
                return token;
            }
        }
    }

    pub fn get_valid_token(&self, token: Option<String>) -> String {
        match token {
            Some(token) if self.token_exists(&token) => token,
            _ => self.generate_token(),
        }
    }

    pub fn session(&mut self, token: String, starting_room: Rc<Room>) -> usize {
        match self.token_store.get(&token) {
            Some(id) => id.to_owned(),
            None => {
                let id = self.id_counter;
                self.id_counter += 1;
                self.token_store.insert(token, id);
                self.sessions.insert(
                    id,
                    User {
                        room: starting_room.clone(),
                        client: None,
                    },
                );
                id
            }
        }
    }

    pub fn token_exists(&self, token: &str) -> bool {
        self.token_store.contains_key(token)
    }

    pub fn get(&'a self, user_id: usize) -> Option<&'a User> {
        self.sessions.get(&user_id)
    }
}

pub struct GameServer {
    user: UserManager,
    starting_room: Rc<Room>,
}

impl GameServer {
    fn new() -> Self {
        GameServer {
            user: UserManager::new(),
            starting_room: Rc::new(Room::WaitingRoom),
        }
    }

    fn handle(&mut self, msg: ServerEvent) {
        match msg {
            ServerEvent::Connect(client, token) => {
                let token = self.user.get_valid_token(token);
                let user_id = self.user.session(token.clone(), self.starting_room.clone());
                let mut client = client.clone();
                rt::spawn(async move {
                    let _ = client.send(ClientEvent::Id(user_id, token)).await;
                });
            }
            ServerEvent::Disconnect(client_id) => {
                println!("User {} disconnected", client_id);
            },
            ServerEvent::Message(packet, client_id) => {

            }
        }
    }

    pub fn start() -> UnboundedSender<ServerEvent> {
        let (tx, mut rx) = mpsc::unbounded();

        rt::Arbiter::new().exec_fn(move || {
            rt::spawn(async move {
                let mut srv = GameServer::new();

                while let Some(msg) = rx.next().await {
                    srv.handle(msg);
                }

                rt::Arbiter::current().stop();
            });
        });

        tx
    }
}
