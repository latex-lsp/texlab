use lsp_types::*;
use texlab::definition::DefinitionResponse;
use texlab::range::RangeExt;
use texlab::scenario::{Scenario, FULL_CAPABILITIES};

pub async fn run(
    scenario: &'static str,
    file: &'static str,
    position: Position,
) -> (Scenario, Vec<Location>) {
    let scenario = format!("definition/{}", scenario);
    let scenario = Scenario::new(&scenario, &FULL_CAPABILITIES).await;
    let identifier = TextDocumentIdentifier::new(scenario.uri(file).into());
    let params = TextDocumentPositionParams::new(identifier, position);
    scenario.open(file).await;
    let definitions = scenario.server.definition(params).await.unwrap();
    let locations = match definitions {
        DefinitionResponse::Locations(locations) => locations,
        DefinitionResponse::LocationLinks(_) => unreachable!(),
    };
    (scenario, locations)
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_citation() {
    let (scenario, definitions) = run("citation", "foo.tex", Position::new(5, 8)).await;
    assert_eq!(
        definitions,
        vec![Location::new(
            scenario.uri("foo.bib").into(),
            Range::new_simple(2, 9, 2, 12)
        )]
    );
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_label() {
    let (scenario, definitions) = run("label", "foo.tex", Position::new(8, 8)).await;
    assert_eq!(
        definitions,
        vec![Location::new(
            scenario.uri("bar.tex").into(),
            Range::new_simple(0, 7, 0, 10)
        )]
    );
}
