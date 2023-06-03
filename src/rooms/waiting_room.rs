use crate::event::Event;

pub fn waiting_room_on(event: Event) {
    match event {
        Event::Connect => println!("A player connected to waiting room"),
        _ => {}
    }
}
