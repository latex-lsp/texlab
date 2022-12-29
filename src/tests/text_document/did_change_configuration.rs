use insta::assert_snapshot;
use lsp_types::{
    notification::{DidChangeConfiguration, Notification, ShowMessage},
    ClientCapabilities, DidChangeConfigurationParams, ShowMessageParams,
};

use crate::tests::client::Client;

#[test]
fn invalid_configuration() {
    let mut client = Client::spawn();
    client.initialize(ClientCapabilities::default(), None);

    client.notify::<DidChangeConfiguration>(DidChangeConfigurationParams {
        settings: serde_json::json!({
            "diagnostics": {
                "allowedPatterns": ["\\"]
            }
        }),
    });

    let result = client.shutdown();
    let message = result
        .incoming
        .notifications
        .into_iter()
        .filter_map(|notification| {
            notification
                .extract::<ShowMessageParams>(ShowMessage::METHOD)
                .ok()
        })
        .find(|params| params.message.contains("configuration"))
        .unwrap()
        .message;

    assert_snapshot!(message);
}
