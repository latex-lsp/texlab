#![feature(await_macro, async_await)]

mod server;
mod types;

pub use self::server::*;
pub use self::types::*;

#[macro_export]
macro_rules! handle_message {
    ($message:ident, $server:ident) => {{
        use jsonrpc::*;

        let handle = async move |message| {
            let message: Message = serde_json::from_str(message).map_err(|_| Error {
                code: ErrorCode::ParseError,
                message: "Could not parse the input".to_owned(),
                data: serde_json::Value::Null,
            })?;

            match message {
                Message::Request(request) => Ok(Some(await!($server.handle_request(request)))),
                Message::Notification(notification) => {
                    await!($server.handle_notification(notification));
                    Ok(None)
                }
                Message::Response(_) => panic!("Unexpected client response"),
            }
        };

        match await!(handle(&$message)) {
            Ok(response) => response,
            Err(error) => Some(Response::new(serde_json::Value::Null, Some(error), None)),
        }
    }};
}
