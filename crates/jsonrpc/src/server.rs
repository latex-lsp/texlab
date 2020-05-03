use super::types::*;
use async_trait::async_trait;
use futures::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;

pub type Result<T> = std::result::Result<T, String>;

#[async_trait]
pub trait RequestHandler {
    async fn handle_request(&self, request: Request) -> Response;

    async fn handle_notification(&self, notification: Notification);
}

#[async_trait]
pub trait Middleware {
    async fn before_message(&self);

    async fn after_message(&self);
}

pub async fn handle_request<'a, H, F, I, O>(request: Request, handler: H) -> Response
where
    H: Fn(I) -> F + Send + Sync + 'a,
    F: Future<Output = Result<O>> + Send,
    I: DeserializeOwned + Send,
    O: Serialize,
{
    let handle = |json| async move {
        let params: I = serde_json::from_value(json).map_err(|_| Error::deserialize_error())?;
        let result = handler(params).await.map_err(Error::internal_error)?;
        Ok(result)
    };

    match handle(request.params).await {
        Ok(result) => Response::result(json!(result), request.id),
        Err(error) => Response::error(error, Some(request.id)),
    }
}

pub async fn handle_notification<'a, H, F, I>(notification: Notification, handler: H)
where
    H: Fn(I) -> F + Send + Sync + 'a,
    F: Future<Output = ()> + Send,
    I: DeserializeOwned + Send,
{
    let error = Error::deserialize_error().message;
    let params = serde_json::from_value(notification.params).expect(&error);
    handler(params).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    const METHOD_NAME: &str = "foo";

    async fn increment(i: i32) -> Result<i32> {
        Ok(i + 1)
    }

    async fn panic(_params: ()) {
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

    #[tokio::test]
    #[should_panic(expected = "success")]
    async fn notification_valid() {
        let notification = setup_notification();
        handle_notification(notification, panic).await;
    }

    #[tokio::test]
    #[should_panic]
    async fn notification_invalid_params() {
        let notification = Notification {
            params: json!(0),
            ..setup_notification()
        };
        handle_notification(notification, panic).await;
    }
}
