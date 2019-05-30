#![feature(async_await)]

use futures::executor::block_on;
use lsp_types::*;
use texlab::scenario::Scenario;

pub async fn run(
    scenario: &'static str,
    file: &'static str,
    position: Position,
) -> (Scenario, Vec<Location>) {
    let scenario = format!("definition/{}", scenario);
    let scenario = Scenario::new(&scenario).await;
    let identifier = TextDocumentIdentifier::new(scenario.uri(file));
    let params = TextDocumentPositionParams::new(identifier, position);
    scenario.open(file).await;
    let definitions = scenario.server.definition(params).await.unwrap();
    (scenario, definitions)
}

#[test]
fn test_citation() {
    block_on(async move {
        let (scenario, definitions) = run("citation", "foo.tex", Position::new(5, 8)).await;
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
        let (scenario, definitions) = run("label", "foo.tex", Position::new(8, 8)).await;
        assert_eq!(
            definitions,
            vec![Location::new(
                scenario.uri("bar.tex"),
                Range::new_simple(0, 7, 0, 10)
            )]
        );
    });
}
