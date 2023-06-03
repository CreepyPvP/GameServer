use crate::event::Event;

use self::waiting_room::waiting_room_on;

mod waiting_room;

pub enum Room {
    WaitingRoom,
}

impl Room {
    pub fn on(&self, event: Event) {
        match self {
            Room::WaitingRoom => waiting_room_on(event),
            _ => {}
        }
    }
}
