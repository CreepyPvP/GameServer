use serde::{Deserialize, Serialize};

pub mod client_message;
pub mod server_message;

#[derive(Serialize, Deserialize)]
pub struct RawPacket {
    pub channel: String,
    pub data: serde_json::Value,
}
