use super::capabilities::CLIENT_FULL_CAPABILITIES;
use super::scenario::Scenario;
use texlab_protocol::*;

pub async fn run_list(
    scenario_short_name: &'static str,
    file: &'static str,
    line: u64,
    character: u64,
) -> (Scenario, Vec<CompletionItem>) {
    let scenario_name = format!("completion/{}", scenario_short_name);
    let scenario = Scenario::new(&scenario_name, false).await;
    scenario.initialize(&CLIENT_FULL_CAPABILITIES).await;
    scenario.open(file).await;

    let params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier::new(scenario.uri(file).into()),
            position: Position::new(line, character),
        },
        context: None,
    };

    let items = scenario
        .server
        .execute_async(|svr| svr.completion(params))
        .await
        .unwrap()
        .items;

    (scenario, items)
}

pub async fn run_empty(
    scenario_short_name: &'static str,
    file: &'static str,
    line: u64,
    character: u64,
) {
    assert!(run_list(scenario_short_name, file, line, character)
        .await
        .1
        .is_empty());
}

pub async fn run_item(
    scenario_short_name: &'static str,
    file: &'static str,
    line: u64,
    character: u64,
    item_name: &'static str,
) -> CompletionItem {
    let (scenario, items) = run_list(scenario_short_name, file, line, character).await;

    let item = items
        .into_iter()
        .find(|item| item.label == item_name)
        .unwrap();

    scenario
        .server
        .execute_async(|svr| svr.completion_resolve(item))
        .await
        .unwrap()
}

pub mod verify {
    use super::*;

    pub fn text_edit(
        item: &CompletionItem,
        start_line: u64,
        start_character: u64,
        end_line: u64,
        end_character: u64,
        text: &str,
    ) {
        assert_eq!(
            item.text_edit,
            Some(TextEdit::new(
                Range::new_simple(start_line, start_character, end_line, end_character),
                text.into()
            ))
        );
    }

    pub fn detail(item: &CompletionItem, detail: &str) {
        assert_eq!(item.detail.as_ref().unwrap(), detail);
    }

    pub fn labels(items: &[CompletionItem], expected_labels: Vec<&'static str>) {
        let mut actual_labels: Vec<&str> = items.iter().map(|item| item.label.as_ref()).collect();
        actual_labels.sort();
        assert_eq!(actual_labels, expected_labels);
    }
}
