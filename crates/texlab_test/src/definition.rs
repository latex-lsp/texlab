use super::capabilities::{CLIENT_FULL_CAPABILITIES, CLIENT_NO_LINK_CAPABILITIES};
use super::scenario::Scenario;
use texlab_protocol::*;

pub async fn run(
    scenario_short_name: &'static str,
    file: &'static str,
    line: u64,
    character: u64,
    capabilities: &ClientCapabilities,
) -> (Scenario, DefinitionResponse) {
    let scenario_name = format!("definition/{}", scenario_short_name);
    let scenario = Scenario::new(&scenario_name, false).await;
    scenario.initialize(capabilities).await;
    scenario.open(file).await;

    let params = TextDocumentPositionParams {
        text_document: TextDocumentIdentifier::new(scenario.uri(file).into()),
        position: Position::new(line, character),
    };

    let response = scenario
        .server
        .execute(|svr| svr.definition(params))
        .await
        .unwrap();

    (scenario, response)
}

pub async fn run_link(
    scenario_short_name: &'static str,
    file: &'static str,
    line: u64,
    character: u64,
) -> (Scenario, Vec<LocationLink>) {
    let (scenario, response) = run(
        scenario_short_name,
        file,
        line,
        character,
        &CLIENT_FULL_CAPABILITIES,
    )
    .await;
    match response {
        DefinitionResponse::LocationLinks(links) => (scenario, links),
        DefinitionResponse::Locations(_) => unreachable!(),
    }
}

pub async fn run_location(
    scenario_short_name: &'static str,
    file: &'static str,
    line: u64,
    character: u64,
) -> (Scenario, Vec<Location>) {
    let (scenario, response) = run(
        scenario_short_name,
        file,
        line,
        character,
        &CLIENT_NO_LINK_CAPABILITIES,
    )
    .await;
    match response {
        DefinitionResponse::LocationLinks(_) => unreachable!(),
        DefinitionResponse::Locations(locations) => (scenario, locations),
    }
}

pub mod verify {
    use super::*;

    pub fn origin_selection_range(
        link: &LocationLink,
        start_line: u64,
        start_character: u64,
        end_line: u64,
        end_character: u64,
    ) {
        assert_eq!(
            link.origin_selection_range,
            Some(Range::new_simple(
                start_line,
                start_character,
                end_line,
                end_character
            ))
        );
    }
}
