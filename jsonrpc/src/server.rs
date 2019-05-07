use crate::types::*;
use futures::prelude::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;

const DESERIALIZE_OBJECT_ERROR: &str = "Could not deserialize parameter object";

pub async fn handle_request<'a, H, F, I, O>(request: Request, handler: H) -> Response
where
    H: Fn(I) -> F + Send + Sync + 'a,
    F: Future<Output = Result<O>> + Send,
    I: DeserializeOwned + Send,
    O: Serialize,
{
    let handle = async move |json| -> std::result::Result<O, Error> {
        let params: I = serde_json::from_value(json).map_err(|_| Error {
            code: ErrorCode::InvalidParams,
            message: String::from(DESERIALIZE_OBJECT_ERROR),
            data: serde_json::Value::Null,
        })?;

        let result = await!(handler(params)).map_err(|message| Error {
            code: ErrorCode::InternalError,
            message,
            data: serde_json::Value::Null,
        })?;

        Ok(result)
    };

    match await!(handle(request.params)) {
        Ok(result) => Response::new(json!(result), None, Some(request.id)),
        Err(error) => Response::new(serde_json::Value::Null, Some(error), Some(request.id)),
    }
}

pub fn handle_notification<'a, H, I>(notification: Notification, handler: H)
where
    H: Fn(I) -> () + Send + Sync + 'a,
    I: DeserializeOwned + Send,
{
    match serde_json::from_value(notification.params) {
        Ok(params) => handler(params),
        Err(_) => panic!(DESERIALIZE_OBJECT_ERROR),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;

    const JSONRPC_VERSION: &str = "2.0";
    const METHOD_NAME: &str = "foo";

    async fn increment(i: i32) -> Result<i32> {
        Ok(i + 1)
    }

    fn panic(_params: ()) {
        panic!("success");
    }

    fn setup_request<T: Serialize>(value: T) -> Request {
        Request {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            params: json!(value),
            method: METHOD_NAME.to_owned(),
            id: 0,
        }
    }

    fn setup_notification() -> Notification {
        Notification {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            method: METHOD_NAME.to_owned(),
            params: json!(()),
        }
    }

    #[test]
    fn test_request_valid() {
        let value = 42;
        let request = setup_request(value);

        let response = block_on(handle_request(request.clone(), increment));
        let expected = Response {
            jsonrpc: request.jsonrpc,
            result: json!(block_on(increment(value)).unwrap()),
            error: None,
            id: Some(request.id),
        };

        assert_eq!(response, expected);
    }

    #[test]
    fn test_request_invalid_params() {
        let request = setup_request((0, 0));

        let response = block_on(handle_request(request.clone(), increment));
        let expected = Response {
            jsonrpc: request.jsonrpc.clone(),
            result: serde_json::Value::Null,
            error: Some(Error {
                code: ErrorCode::InvalidParams,
                message: DESERIALIZE_OBJECT_ERROR.to_owned(),
                data: serde_json::Value::Null,
            }),
            id: Some(request.id),
        };

        assert_eq!(response, expected);
    }

    #[test]
    #[should_panic(expected = "success")]
    fn test_notification_valid() {
        let notification = setup_notification();
        handle_notification(notification, panic);
    }

    #[test]
    #[should_panic]
    fn test_notification_invalid_params() {
        let notification = setup_notification();
        let notification = Notification {
            params: json!(0),
            ..notification
        };

        handle_notification(notification, panic);
    }
}
