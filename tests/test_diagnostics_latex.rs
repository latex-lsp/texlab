pub mod support;

use lsp_types::*;
use std::sync::Arc;
use support::*;
use tex::DistributionKind::{Miktex, Texlive};
use texlab::protocol_types::LatexLintOptions;

#[tokio::test]
async fn disabled() {
    let _ = tex::with_distro(&[Texlive, Miktex], |distro| {
        async move {
            let scenario = Scenario::new("diagnostics/latex", Arc::new(distro));
            scenario.client.options.lock().await.latex_lint = Some(LatexLintOptions {
                on_change: Some(false),
                on_save: Some(false),
            });

            scenario
                .initialize(&capabilities::CLIENT_FULL_CAPABILITIES)
                .await;
            scenario.open("disabled.tex").await;
            scenario
                .client
                .verify_no_diagnostics(&scenario.uri("disabled.tex"))
                .await;
        }
    })
    .await;
}

#[tokio::test]
async fn on_open() {
    let _ = tex::with_distro(&[Texlive, Miktex], |distro| {
        async move {
            let scenario = Scenario::new("diagnostics/latex", Arc::new(distro));
            scenario.client.options.lock().await.latex_lint = Some(LatexLintOptions {
                on_change: Some(false),
                on_save: Some(true),
            });

            scenario
                .initialize(&capabilities::CLIENT_FULL_CAPABILITIES)
                .await;
            scenario.open("on_open.tex").await;
            {
                let diagnostics_by_uri = scenario.client.diagnostics_by_uri.lock().await;
                let diagnostics = &diagnostics_by_uri[&scenario.uri("on_open.tex")];
                assert_eq!(diagnostics.len(), 1);
                assert_eq!(diagnostics[0].message, "Command terminated with space.");
            }
        }
    })
    .await;
}

#[tokio::test]
async fn on_save() {
    let _ = tex::with_distro(&[Texlive, Miktex], |distro| {
        async move {
            let scenario = Scenario::new("diagnostics/latex", Arc::new(distro));
            scenario.client.options.lock().await.latex_lint = Some(LatexLintOptions {
                on_change: Some(false),
                on_save: Some(true),
            });

            scenario
                .initialize(&capabilities::CLIENT_FULL_CAPABILITIES)
                .await;
            scenario.open("on_save.tex").await;
            let uri = scenario.uri("on_save.tex");
            scenario.client.verify_no_diagnostics(&uri).await;

            let text_document = VersionedTextDocumentIdentifier::new(uri.clone().into(), 0);
            let content_change = TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "\\foo\n".into(),
            };
            let did_change_params = DidChangeTextDocumentParams {
                text_document,
                content_changes: vec![content_change],
            };
            scenario
                .server
                .execute(|svr| svr.did_change(did_change_params))
                .await;
            scenario.client.verify_no_diagnostics(&uri).await;

            let text_document = TextDocumentIdentifier::new(uri.clone().into());
            let did_save_params = DidSaveTextDocumentParams { text_document };
            scenario
                .server
                .execute(|svr| svr.did_save(did_save_params))
                .await;
            {
                let diagnostics_by_uri = scenario.client.diagnostics_by_uri.lock().await;
                let diagnostics = &diagnostics_by_uri[&uri];
                assert_eq!(diagnostics.len(), 1);
                assert_eq!(diagnostics[0].message, "Command terminated with space.");
            }
        }
    })
    .await;
}

#[tokio::test]
async fn on_change() {
    let _ = tex::with_distro(&[Texlive, Miktex], |distro| {
        async move {
            let scenario = Scenario::new("diagnostics/latex", Arc::new(distro));
            scenario.client.options.lock().await.latex_lint = Some(LatexLintOptions {
                on_change: Some(true),
                on_save: Some(true),
            });

            scenario
                .initialize(&capabilities::CLIENT_FULL_CAPABILITIES)
                .await;
            scenario.open("on_change.tex").await;
            let uri = scenario.uri("on_change.tex");
            scenario.client.verify_no_diagnostics(&uri).await;

            let text_document = VersionedTextDocumentIdentifier::new(uri.clone().into(), 0);
            let content_change = TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "\\foo\n".into(),
            };
            let did_change_params = DidChangeTextDocumentParams {
                text_document,
                content_changes: vec![content_change],
            };
            scenario
                .server
                .execute(|svr| svr.did_change(did_change_params))
                .await;
            {
                let diagnostics_by_uri = scenario.client.diagnostics_by_uri.lock().await;
                let diagnostics = &diagnostics_by_uri[&uri];
                assert_eq!(diagnostics.len(), 1);
                assert_eq!(diagnostics[0].message, "Command terminated with space.");
            }
        }
    })
    .await;
}
