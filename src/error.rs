use futures::channel::mpsc::SendError;
use ntex::{
    http::StatusCode,
    web::{self, HttpRequest, HttpResponse, WebResponseError},
};
use redis::RedisError;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum AppError {
    WebError(String),
    JsonError(String),
    IoError(String),
    RedisError(String),
    SendError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Error]: {}",
            match self {
                Self::WebError(msg) => format!("WebError: {}", msg),
                Self::JsonError(msg) => format!("JsonError: {}", msg),
                Self::IoError(msg) => format!("IoError: {}", msg),
                Self::RedisError(msg) => format!("RedisError: {}", msg),
                Self::SendError(msg) => format!("SendError: {}", msg),
            }
        )
    }
}

impl From<serde_json::error::Error> for AppError {
    fn from(value: serde_json::error::Error) -> Self {
        Self::JsonError(value.to_string())
    }
}

impl From<web::Error> for AppError {
    fn from(value: web::Error) -> Self {
        Self::WebError(value.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<RedisError> for AppError {
    fn from(value: RedisError) -> Self {
        Self::RedisError(value.to_string())
    }
}

impl From<SendError> for AppError {
    fn from(value: SendError) -> Self {
        Self::SendError(value.to_string())
    }
}

impl WebResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(&self)
    }
}

impl std::error::Error for AppError {}
