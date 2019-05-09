use serde::{Deserialize, Serialize};
use serde_repr::*;

pub const PROTOCOL_VERSION: &str = "2.0";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize_repr, Serialize_repr)]
#[repr(i32)]
pub enum ErrorCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    ServerNotInitialized = -32002,
    UnknownErrorCode = -32001,
    RequestCancelled = -32800,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,

    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Request {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: i32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Response {
    pub jsonrpc: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Error>,

    pub id: Option<i32>,
}

impl Response {
    pub fn result(result: serde_json::Value, id: i32) -> Self {
        Response {
            jsonrpc: PROTOCOL_VERSION.to_owned(),
            result: Some(result),
            error: None,
            id: Some(id),
        }
    }

    pub fn error(error: Error, id: Option<i32>) -> Self {
        Response {
            jsonrpc: PROTOCOL_VERSION.to_owned(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Notification {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Notification(Notification),
    Response(Response),
}

pub type Result<T> = std::result::Result<T, String>;
