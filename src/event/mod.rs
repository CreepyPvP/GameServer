// #[derive(Serialize, Deserialize)]
// pub struct RawPacket {
//     pub channel: String,
//     pub data: serde_json::Value,
// }

// pub enum Event {
//     Connect,
//     Join,
//
//     // Outgoing
//     SetAuthToken { token: String },
// }
//
// impl Event {
//     pub fn parse(data: RawPacket) -> Result<Event, GameError> {
//         match data.channel.as_str() {
//             _ => Err(GameError {
//                 context: "Parsing Packet".to_string(),
//                 msg: "Unknown channel".to_string(),
//             }),
//         }
//     }
//
//     pub fn stringfy(&self) -> Result<String, GameError> {
//         let (channel, data): (String, serde_json::Value) = match self {
//             Self::SetAuthToken { token } => {
//                 Ok(("set_auth_token".to_string(), json!({ "token": token })))
//             }
//             _ => Err(GameError {
//                 context: "Serializing Packet".to_string(),
//                 msg: "This packet is not serializable".to_string(),
//             }),
//         }?;
//
//         let raw_packet = RawPacket { channel, data };
//         let result = serde_json::to_string(&raw_packet)?;
//         Ok(result)
//     }
// }

pub mod client_message;
pub mod server_message;
