use super::capabilities::CLIENT_FULL_CAPABILITIES;
use super::scenario::Scenario;
use texlab_protocol::*;

pub async fn run_hierarchical(file: &'static str) -> Vec<DocumentSymbol> {
    let scenario = Scenario::new("symbol/hierarchical", false).await;
    scenario.initialize(&CLIENT_FULL_CAPABILITIES).await;
    scenario.open(file).await;
    let params = DocumentSymbolParams {
        text_document: TextDocumentIdentifier::new(scenario.uri(file).into()),
    };

    let response = scenario
        .server
        .execute_async(|svr| svr.document_symbol(params))
        .await
        .unwrap();

    match response {
        DocumentSymbolResponse::Nested(symbols) => symbols,
        DocumentSymbolResponse::Flat(_) => unreachable!(),
    }
}

pub async fn run_workspace(query: &'static str) -> (Scenario, Vec<SymbolInformation>) {
    let scenario = Scenario::new("symbol/workspace", false).await;
    scenario.initialize(&CLIENT_FULL_CAPABILITIES).await;
    scenario.open("foo.tex").await;
    scenario.open("bar.bib").await;
    let params = WorkspaceSymbolParams {
        query: query.into(),
    };
    let symbols = scenario
        .server
        .execute_async(|svr| svr.workspace_symbol(params))
        .await
        .unwrap();

    (scenario, symbols)
}

pub mod verify {
    use super::*;

    pub fn symbol(
        symbol: &DocumentSymbol,
        name: &str,
        detail: Option<&str>,
        selection_range: Range,
        range: Range,
    ) {
        assert_eq!(symbol.name, name);
        assert_eq!(symbol.detail.as_ref().map(AsRef::as_ref), detail);
        assert_eq!(symbol.selection_range, selection_range);
        assert_eq!(symbol.range, range);
    }

    pub fn symbol_info(
        symbol: &SymbolInformation,
        scenario: &Scenario,
        file: &str,
        name: &str,
        start_line: u64,
        start_character: u64,
        end_line: u64,
        end_character: u64,
    ) {
        assert_eq!(symbol.name, name);
        let range = Range::new_simple(start_line, start_character, end_line, end_character);
        assert_eq!(
            symbol.location,
            Location::new(scenario.uri(file).into(), range)
        );
    }
}
