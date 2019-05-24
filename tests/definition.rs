#![feature(await_macro, async_await)]

mod common;

use crate::common::Scenario;
use futures::executor::block_on;
use lsp_types::*;

pub async fn run(
    scenario: &'static str,
    file: &'static str,
    position: Position,
) -> (Scenario, Vec<Location>) {
    let scenario = format!("definition/{}", scenario);
    let scenario = await!(Scenario::new(&scenario));
    let identifier = TextDocumentIdentifier::new(scenario.uri(file));
    let params = TextDocumentPositionParams::new(identifier, position);
    await!(scenario.open(file));
    let definitions = await!(scenario.server.definition(params)).unwrap();
    (scenario, definitions)
}

#[test]
fn test_citation() {
    block_on(async move {
        let (scenario, definitions) = await!(run("citation", "foo.tex", Position::new(5, 8)));
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
        let (scenario, definitions) = await!(run("label", "foo.tex", Position::new(8, 8)));
        assert_eq!(
            definitions,
            vec![Location::new(
                scenario.uri("bar.tex"),
                Range::new_simple(0, 7, 0, 10)
            )]
        );
    });
}
