use serde::{Deserialize, Serialize};
use serde_repr::*;

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

    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    pub result: serde_json::Value,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Error>,

    pub id: Option<i32>,
}

impl Response {
    pub fn new(result: serde_json::Value, error: Option<Error>, id: Option<i32>) -> Self {
        Response {
            jsonrpc: String::from("2.0"),
            result,
            error,
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
    Response(Response),
    Notification(Notification),
}

pub type Result<T> = std::result::Result<T, String>;
