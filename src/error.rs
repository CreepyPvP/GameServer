use ntex::web;

#[derive(Debug)]
pub struct GameError {
    pub context: String,
    pub msg: String,
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Error]: {}: {}", self.context, self.msg)
    }
}

impl From<serde_json::error::Error> for GameError {
    fn from(value: serde_json::error::Error) -> Self {
        GameError {
            context: "JsonError".to_string(),
            msg: value.to_string(),
        }
    }
}

impl From<web::Error> for GameError {
    fn from(value: web::Error) -> Self {
        GameError {
            context: "WebError".to_string(),
            msg: value.to_string(),
        }
    }
}

impl std::error::Error for GameError {}
