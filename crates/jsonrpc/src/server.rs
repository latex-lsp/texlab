use crate::types::*;
use futures::prelude::*;
use futures_boxed::boxed;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;

pub type Result<T> = std::result::Result<T, String>;

pub trait RequestHandler {
    #[boxed]
    async fn handle_request(&self, request: Request) -> Response;

    fn handle_notification(&self, notification: Notification);
}

pub trait Middleware {
    #[boxed]
    async fn before_message(&self);

    #[boxed]
    async fn after_message(&self);
}

pub async fn handle_request<'a, H, F, I, O>(request: Request, handler: H) -> Response
where
    H: Fn(I) -> F + Send + Sync + 'a,
    F: Future<Output = Result<O>> + Send,
    I: DeserializeOwned + Send,
    O: Serialize,
{
    let handle = |json| {
        async move {
            let params: I = serde_json::from_value(json).map_err(|_| Error::deserialize_error())?;
            let result = handler(params).await.map_err(Error::internal_error)?;
            Ok(result)
        }
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
            id: Id::Number(0),
        }
    }

    fn setup_notification() -> Notification {
        Notification {
            jsonrpc: PROTOCOL_VERSION.to_owned(),
            method: METHOD_NAME.to_owned(),
            params: json!(()),
        }
    }

    #[tokio::test]
    async fn request_valid() {
        let value = 42;
        let request = setup_request(value);

        let response = handle_request(request.clone(), increment).await;
        let expected = Response {
            jsonrpc: request.jsonrpc,
            result: Some(json!(increment(value).await.unwrap())),
            error: None,
            id: Some(request.id),
        };

        assert_eq!(response, expected);
    }

    #[tokio::test]
    async fn request_invalid_params() {
        let request = setup_request((0, 0));

        let response = handle_request(request.clone(), increment).await;
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
    fn notification_valid() {
        let notification = setup_notification();
        handle_notification(notification, panic);
    }

    #[test]
    #[should_panic]
    fn notification_invalid_params() {
        let notification = setup_notification();
        let notification = Notification {
            params: json!(0),
            ..notification
        };

        handle_notification(notification, panic);
    }
}
