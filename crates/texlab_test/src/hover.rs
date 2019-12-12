use super::capabilities::CLIENT_FULL_CAPABILITIES;
use super::scenario::Scenario;
use texlab_protocol::*;

pub async fn run(
    scenario_short_name: &'static str,
    file: &'static str,
    line: u64,
    character: u64,
) -> Option<HoverContents> {
    let scenario_name = format!("hover/{}", scenario_short_name);
    let scenario = Scenario::new(&scenario_name, false).await;
    scenario.initialize(&CLIENT_FULL_CAPABILITIES).await;
    scenario.open(file).await;
    let identifier = TextDocumentIdentifier::new(scenario.uri(file).into());
    let params = TextDocumentPositionParams::new(identifier, Position::new(line, character));
    scenario
        .server
        .execute_async(|svr| svr.hover(params))
        .await
        .unwrap()
        .map(|hover| hover.contents)
}
