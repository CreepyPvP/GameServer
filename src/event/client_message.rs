use serde::{Serialize, Deserialize};

use super::RawPacket;

pub enum ClientEvent {
    Id(usize, String),
    Message(ClientMessage)
}


// outgoing

#[derive(Serialize, Deserialize)]
struct TestMessageData {
    msg: String,
}

pub enum ClientMessage {
    Test(TestMessageData)
}

impl ClientMessage {
    pub fn stringfy(self) -> Result<String, serde_json::Error> {
        let (channel, data) = match self {
            Self::Test(data) => ("test", serde_json::to_value(data)?),
        };
        let raw = RawPacket {
            channel: channel.to_string(), data
        };

        serde_json::to_string(&raw) 
    } 
}
