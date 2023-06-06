use std::{collections::HashMap, sync::Arc};
use futures::{channel::mpsc::{self, UnboundedSender}, StreamExt};
use ntex::rt;
use crate::{rooms::Room, event::server_message::ServerMessage};

pub struct User {
    pub room: Arc<Room>,
}

pub struct UserManager {
    starting_room: Arc<Room>,
    id_counter: usize,

    sessions: HashMap<usize, User>,
    token_store: HashMap<String, usize>,
}

impl<'a> UserManager {
    fn new(starting_room: Arc<Room>) -> Self {
        UserManager {
            id_counter: 0,
            starting_room,
            sessions: HashMap::new(),
            token_store: HashMap::new(),
        }
    }

    pub fn get_or_create(&mut self, token: String) -> usize {
        match self.token_store.get(&token) {
            Some(id) => id.to_owned(),
            None => {
                let id = self.id_counter;
                self.id_counter += 1;
                self.token_store.insert(token, id);
                self.sessions.insert(
                    id,
                    User {
                        room: self.starting_room.clone(),
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
    pub user: UserManager,
}

impl GameServer {
    fn new() -> Self {
        GameServer {
            user: UserManager::new(Arc::new(Room::WaitingRoom)),
        }
    }

    pub fn start() -> UnboundedSender<ServerMessage> {
        let (tx, mut rx) = mpsc::unbounded();

        rt::Arbiter::new().exec_fn(move || {
            rt::spawn(async move {
                let mut srv = GameServer::new();

                while let Some(msg) = rx.next().await {
                    // Handle message
                }

                rt::Arbiter::current().stop();
            });
        });

        tx
    }
}
