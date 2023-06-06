use futures::channel::mpsc::UnboundedSender;
use serde::{Serialize, Deserialize};

use super::{client_message::ClientEvent, RawPacket};

pub enum ServerEvent {
    Connect(UnboundedSender<ClientEvent>, Option<String>),
    Disconnect(usize),
    Message(ServerMessage, usize),
}



// inbound

#[derive(Serialize, Deserialize)]
struct CreateLobbyData {
    name: String,
}

#[derive(Serialize, Deserialize)]
struct TestData {
    secret_number: usize,
}

pub enum ServerMessage {
    CreateLobby,
    Test(TestData),
}

impl ServerMessage {
    pub fn parse(msg: String) -> Option<Self> {
        let raw: RawPacket = match serde_json::from_str(&msg) {
            Ok(raw) => raw,
            Err(_) => { return None; }
        };

        let parse = || -> Result<Option<ServerMessage>, serde_json::Error> {
            Ok(match raw.channel.as_str() {
                "test" => Some(Self::Test(serde_json::from_value(raw.data)?)),
                "create_lobby" => Some(Self::CreateLobby),
                _ => None,
            })
        };

        match parse() {
            Ok(value) => value,
            Err(_) => None,
        }
    }
}
