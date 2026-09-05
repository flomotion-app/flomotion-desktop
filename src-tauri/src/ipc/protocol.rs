use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const COMMAND_STATUS: &str = "status";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Request {
    pub command: String,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Response {
    pub ok: bool,
    #[serde(default)]
    pub result: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Request {
    pub fn new(command: &str, payload: Value) -> Self {
        Self { command: command.to_string(), payload }
    }

    pub fn status() -> Self {
        Self::new(COMMAND_STATUS, Value::Null)
    }
}

impl Response {
    pub fn success(result: Value) -> Self {
        Self { ok: true, result, error: None }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self { ok: false, result: Value::Null, error: Some(message.into()) }
    }
}

pub trait RequestHandler: Send + Sync {
    fn handle(&self, request: Request) -> Response;
}

pub trait Transport: Send + Sync {
    fn send(&self, request: Request) -> crate::error::Result<Response>;
}
