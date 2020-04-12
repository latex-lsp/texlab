use serde::{Deserialize, Serialize};
use serde_repr::*;

pub const PROTOCOL_VERSION: &str = "2.0";

#[derive(Debug, Eq, Hash, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Id {
    Number(u64),
    String(String),
}

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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Error {
    pub fn parse_error() -> Self {
        Self {
            code: ErrorCode::ParseError,
            message: "Could not parse the input".to_owned(),
            data: None,
        }
    }

    pub fn method_not_found_error() -> Self {
        Self {
            code: ErrorCode::MethodNotFound,
            message: "Method not found".to_owned(),
            data: None,
        }
    }

    pub fn deserialize_error() -> Self {
        Self {
            code: ErrorCode::InvalidParams,
            message: "Could not deserialize parameter object".to_owned(),
            data: None,
        }
    }

    pub fn internal_error(message: String) -> Self {
        Self {
            code: ErrorCode::InternalError,
            message,
            data: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Request {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: Id,
}

impl Request {
    pub fn new(method: String, params: serde_json::Value, id: Id) -> Self {
        Self {
            jsonrpc: PROTOCOL_VERSION.to_owned(),
            method,
            params,
            id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Response {
    pub jsonrpc: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Error>,

    pub id: Option<Id>,
}

impl Response {
    pub fn result(result: serde_json::Value, id: Id) -> Self {
        Self {
            jsonrpc: PROTOCOL_VERSION.to_owned(),
            result: Some(result),
            error: None,
            id: Some(id),
        }
    }

    pub fn error(error: Error, id: Option<Id>) -> Self {
        Self {
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

impl Notification {
    pub fn new(method: String, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: PROTOCOL_VERSION.to_owned(),
            method,
            params,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Notification(Notification),
    Response(Response),
}
