use super::capabilities::CLIENT_FULL_CAPABILITIES;
use super::scenario::Scenario;
use std::cmp::Reverse;
use texlab_protocol::*;

pub async fn run(file: &'static str) -> Vec<FoldingRange> {
    let scenario = Scenario::new("folding", false).await;
    scenario.initialize(&CLIENT_FULL_CAPABILITIES).await;
    scenario.open(file).await;
    let params = FoldingRangeParams {
        text_document: TextDocumentIdentifier::new(scenario.uri(file).into()),
    };

    let mut foldings = scenario
        .server
        .execute_async(|svr| svr.folding_range(params))
        .await
        .unwrap();

    foldings.sort_by_key(|folding| {
        let start = Position::new(folding.start_line, folding.start_character.unwrap());
        let end = Position::new(folding.end_line, folding.end_character.unwrap());
        (start, Reverse(end))
    });
    foldings
}
