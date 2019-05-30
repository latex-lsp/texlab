use crate::types::*;
use futures::future::BoxFuture;
use futures::prelude::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;

pub type Result<T> = std::result::Result<T, String>;

pub trait RequestHandler {
    fn handle_request(&self, request: Request) -> BoxFuture<'_, Response>;

    fn handle_notification(&self, notification: Notification);
}

pub trait ActionHandler {
    fn execute_actions(&self) -> BoxFuture<'_, ()>;
}

pub async fn handle_request<'a, H, F, I, O>(request: Request, handler: H) -> Response
where
    H: Fn(I) -> F + Send + Sync + 'a,
    F: Future<Output = Result<O>> + Send,
    I: DeserializeOwned + Send,
    O: Serialize,
{
    let handle = async move |json| -> std::result::Result<O, Error> {
        let params: I = serde_json::from_value(json).map_err(|_| Error::deserialize_error())?;
        let result = handler(params).await.map_err(Error::internal_error)?;
        Ok(result)
    };

    match handle(request.params).await {
        Ok(result) => Response::result(json!(result), request.id),
        Err(error) => Response::error(error, Some(request.id)),
    }
}

pub fn handle_notification<'a, H, I>(notification: Notification, handler: H)
where
    H: Fn(I) -> () + Send + Sync + 'a,
    I: DeserializeOwned + Send,
{
    let params =
        serde_json::from_value(notification.params).expect(&Error::deserialize_error().message);
    handler(params);
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;

    const METHOD_NAME: &str = "foo";

    async fn increment(i: i32) -> Result<i32> {
        Ok(i + 1)
    }

    fn panic(_params: ()) {
        panic!("success");
    }

    fn setup_request<T: Serialize>(value: T) -> Request {
        Request {
            jsonrpc: PROTOCOL_VERSION.to_owned(),
            params: json!(value),
            method: METHOD_NAME.to_owned(),
            id: 0,
        }
    }

    fn setup_notification() -> Notification {
        Notification {
            jsonrpc: PROTOCOL_VERSION.to_owned(),
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
            result: Some(json!(block_on(increment(value)).unwrap())),
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
            result: None,
            error: Some(Error::deserialize_error()),
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
