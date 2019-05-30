#![feature(await_macro, async_await)]

mod common;

use crate::common::Scenario;
use futures::executor::block_on;
use jsonrpc::server::ActionHandler;
use lsp_types::*;

async fn run_completion(
    scenario: &Scenario,
    file: &'static str,
    position: Position,
) -> Vec<CompletionItem> {
    let params = CompletionParams {
        text_document: TextDocumentIdentifier::new(scenario.uri(file)),
        position,
        context: None,
    };
    await!(scenario.server.completion(params)).unwrap().items
}

#[test]
fn test_did_change() {
    block_on(async move {
        let scenario = await!(Scenario::new("synchronization/did_change"));
        await!(scenario.open("foo.tex"));
        assert_eq!(
            await!(run_completion(&scenario, "foo.tex", Position::new(0, 1))).len() > 0,
            false
        );

        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier::new(scenario.uri("foo.tex"), 0),
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "\\".to_owned(),
            }],
        };
        scenario.server.did_change(params);
        await!(scenario.server.execute_actions());
        assert_eq!(
            await!(run_completion(&scenario, "foo.tex", Position::new(0, 1))).len() > 0,
            true
        );
    });
}

#[test]
fn test_indexing() {
    block_on(async move {
        let scenario = await!(Scenario::new("synchronization/did_change"));
        await!(scenario.open("foo.tex"));

        let mut path = scenario.directory.path().to_owned();
        path.push("bar.tex");
        std::fs::write(&path, "\\foo").unwrap();

        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier::new(scenario.uri("foo.tex"), 0),
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "\\fo\n\\include{bar}".to_owned(),
            }],
        };
        scenario.server.did_change(params);
        await!(scenario.server.execute_actions());
        let items = await!(run_completion(&scenario, "foo.tex", Position::new(0, 1)));
        assert_eq!(items.iter().any(|item| item.label == "foo"), true);
    });
}
