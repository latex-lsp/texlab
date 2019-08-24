use lsp_types::*;
use std::collections::HashMap;
use texlab::formatting::bibtex::BibtexFormattingOptions;
use texlab::range::RangeExt;
use texlab::scenario::{Scenario, FULL_CAPABILITIES};

pub async fn run(
    scenario: &'static str,
    file: &'static str,
    options: Option<BibtexFormattingOptions>,
) -> (Scenario, Vec<TextEdit>) {
    let scenario = format!("formatting/{}", scenario);
    let scenario = Scenario::new(&scenario, &FULL_CAPABILITIES).await;
    scenario.open(file).await;
    scenario.client.options.lock().await.bibtex_formatting = options;

    let params = DocumentFormattingParams {
        text_document: TextDocumentIdentifier::new(scenario.uri(file).into()),
        options: FormattingOptions {
            tab_size: 4,
            insert_spaces: true,
            properties: HashMap::new(),
        },
    };
    let edits = scenario.server.formatting(params).await.unwrap();
    (scenario, edits)
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_bibtex_entry_default() {
    let (scenario, edits) = run("bibtex/default", "foo.bib", None).await;
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].new_text, scenario.read("bar.bib").await);
    assert_eq!(edits[0].range, Range::new_simple(0, 0, 0, 52));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_bibtex_entry_infinite_line_length() {
    let (scenario, edits) = run(
        "bibtex/infinite_line_length",
        "foo.bib",
        Some(BibtexFormattingOptions {
            line_length: Some(0),
        }),
    )
    .await;
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].new_text, scenario.read("bar.bib").await);
    assert_eq!(edits[0].range, Range::new_simple(0, 0, 0, 149));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_latex() {
    let (_, edits) = run("latex", "foo.tex", None).await;
    assert!(edits.is_empty());
}
