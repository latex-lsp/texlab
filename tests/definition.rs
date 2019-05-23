#![feature(await_macro, async_await)]

mod common;

use crate::common::Scenario;
use futures::executor::block_on;
use lsp_types::*;

#[test]
fn test_citation() {
    block_on(async move {
        let scenario = await!(Scenario::new("definition_citation"));
        let params = TextDocumentPositionParams::new(
            TextDocumentIdentifier::new(scenario.uri("foo.tex")),
            Position::new(5, 8),
        );
        await!(scenario.open("foo.tex", "latex"));
        let definitions = await!(scenario.server.definition(params)).unwrap();
        assert_eq!(
            definitions,
            vec![Location::new(
                scenario.uri("foo.bib"),
                Range::new_simple(2, 9, 2, 12)
            )]
        );
    });
}

#[test]
fn test_label() {
    block_on(async move {
        let scenario = await!(Scenario::new("definition_label"));
        let params = TextDocumentPositionParams::new(
            TextDocumentIdentifier::new(scenario.uri("foo.tex")),
            Position::new(8, 8),
        );
        await!(scenario.open("foo.tex", "latex"));
        let definitions = await!(scenario.server.definition(params)).unwrap();
        assert_eq!(
            definitions,
            vec![Location::new(
                scenario.uri("bar.tex"),
                Range::new_simple(0, 7, 0, 10)
            )]
        );
    });
}
