use futures::channel::mpsc::UnboundedSender;

use super::client_message::ClientMessage;

pub enum ServerMessage {
    Connect(UnboundedSender<ClientMessage>, Option<String>),
    Disconnect(usize),
}
