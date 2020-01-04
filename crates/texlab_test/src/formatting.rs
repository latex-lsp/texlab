use super::capabilities::CLIENT_FULL_CAPABILITIES;
use super::scenario::Scenario;
use std::collections::HashMap;
use texlab_protocol::*;

pub async fn run_bibtex(
    file: &'static str,
    options: Option<BibtexFormattingOptions>,
) -> (Scenario, Vec<TextEdit>) {
    let scenario = Scenario::new("formatting/bibtex", false).await;
    scenario.initialize(&CLIENT_FULL_CAPABILITIES).await;
    scenario.open(file).await;
    {
        *scenario.client.options.lock().await = Options {
            bibtex: Some(BibtexOptions {
                formatting: options,
            }),
            latex: None,
        };
    }

    let params = DocumentFormattingParams {
        text_document: TextDocumentIdentifier::new(scenario.uri(file).into()),
        options: FormattingOptions {
            tab_size: 4,
            insert_spaces: true,
            properties: HashMap::new(),
        },
    };

    let edits = scenario
        .server
        .execute_async(|svr| svr.formatting(params))
        .await
        .unwrap();
    (scenario, edits)
}
