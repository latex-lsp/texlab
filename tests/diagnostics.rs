#![feature(async_await)]

use jsonrpc::server::ActionHandler;
use lsp_types::*;
use texlab::diagnostics::{BibtexErrorCode, LatexLintOptions};
use texlab::scenario::Scenario;

#[runtime::test(runtime_tokio::Tokio)]
async fn test_lint_latex() {
    let scenario = Scenario::new("diagnostics/lint").await;
    scenario.open("foo.tex").await;
    {
        let mut options = scenario.client.options.lock().await;
        options.latex_lint = Some(LatexLintOptions {
            on_save: Some(true),
        });
    }
    let identifier = TextDocumentIdentifier::new(scenario.uri("foo.tex"));
    scenario.server.did_save(DidSaveTextDocumentParams {
        text_document: identifier,
    });
    scenario.server.execute_actions().await;
    scenario.server.stop_scanning().await;
    let diagnostics_by_uri = scenario.client.diagnostics_by_uri.lock().await;
    let diagnostics = diagnostics_by_uri.get(&scenario.uri("foo.tex")).unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        "Wrong length of dash may have been used."
    );
    assert_eq!(diagnostics[0].range.start.line, 4);
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_lint_latex_disabled() {
    let scenario = Scenario::new("diagnostics/lint").await;
    scenario.open("foo.tex").await;
    let identifier = TextDocumentIdentifier::new(scenario.uri("foo.tex"));
    scenario.server.did_save(DidSaveTextDocumentParams {
        text_document: identifier,
    });
    scenario.server.execute_actions().await;
    scenario.server.stop_scanning().await;
    let diagnostics_by_uri = scenario.client.diagnostics_by_uri.lock().await;
    let diagnostics = diagnostics_by_uri.get(&scenario.uri("foo.tex")).unwrap();
    assert!(diagnostics.is_empty());
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_lint_bibtex() {
    let scenario = Scenario::new("diagnostics/lint").await;
    scenario.open("foo.bib").await;
    scenario.server.stop_scanning().await;
    let diagnostics_by_uri = scenario.client.diagnostics_by_uri.lock().await;
    let diagnostics = diagnostics_by_uri.get(&scenario.uri("foo.bib")).unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        BibtexErrorCode::MissingBeginBrace.message()
    );
    assert_eq!(diagnostics[0].range.start.line, 0);
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_build() {
    let scenario = Scenario::new("diagnostics/build").await;
    scenario.open("foo.tex").await;
    scenario
        .server
        .did_change_watched_files(DidChangeWatchedFilesParams {
            changes: vec![FileEvent {
                uri: scenario.uri("foo.log"),
                typ: FileChangeType::Changed,
            }],
        });
    scenario.server.execute_actions().await;
    scenario.server.stop_scanning().await;

    let diagnostics_by_uri = scenario.client.diagnostics_by_uri.lock().await;
    let diagnostics = diagnostics_by_uri.get(&scenario.uri("foo.tex")).unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "Undefined control sequence.");
    assert_eq!(diagnostics[0].range.start.line, 3);
}
