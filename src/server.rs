use std::{collections::HashMap, sync::Arc};

use crate::rooms::Room;

struct User {
    room: Arc<Room>,
}

struct UserManager {
    starting_room: Arc<Room>,
    id_counter: usize,

    sessions: HashMap<usize, User>,
    token_store: HashMap<String, usize>,
}

impl UserManager {
    fn new(starting_room: Arc<Room>) -> Self {
        UserManager { 
            id_counter: 0,
            starting_room,
            sessions: HashMap::new(), 
            token_store: HashMap::new() 
        }
    }

    fn get_or_create(&mut self, token: String) -> usize {
        match self.token_store.get(&token) {
            Some(id) => id.to_owned(),
            None => {
                let id = self.id_counter;
                self.id_counter += 1;
                self.token_store.insert(token, id);
                self.sessions.insert(id, User {
                    room: self.starting_room.clone()
                });
                id
            }
        }
    }
}

pub struct GameServer {
    user: UserManager
}

impl GameServer {
    pub fn new() -> Self {
        GameServer {
            user: UserManager::new(Arc::new(Room::WaitingRoom))
        }
    }

    pub fn get_or_create_user(&mut self, token: String) -> usize {
        self.user.get_or_create(token)
    }

}
