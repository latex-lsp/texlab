use texlab::diagnostics::BibtexErrorCode;
use texlab_protocol::*;
use texlab_test::{Scenario, CLIENT_FULL_CAPABILITIES};

#[tokio::test]
async fn did_change_update() {
    let scenario = Scenario::new("diagnostics/bibtex", false).await;
    scenario.initialize(&CLIENT_FULL_CAPABILITIES).await;
    scenario.open("foo.bib").await;
    {
        let diagnostics_by_uri = scenario.client.diagnostics_by_uri.lock().await;
        let diagnostics = &diagnostics_by_uri[&scenario.uri("foo.bib")];
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message,
            BibtexErrorCode::MissingBeginBrace.message()
        );
        assert_eq!(diagnostics[0].range.start.line, 0);
    }
    let params = DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier::new(scenario.uri("foo.bib").into(), 0),
        content_changes: vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "@article{foo,}\n".into(),
        }],
    };
    scenario.server.execute(|svr| svr.did_change(params)).await;
    {
        let diagnostics_by_uri = scenario.client.diagnostics_by_uri.lock().await;
        let diagnostics = &diagnostics_by_uri[&scenario.uri("foo.bib")];
        assert_eq!(diagnostics.len(), 0);
    }
}
