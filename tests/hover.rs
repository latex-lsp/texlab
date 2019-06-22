#![feature(async_await)]

use lsp_types::*;
use std::borrow::Cow;
use texlab::data::language::LANGUAGE_OPTIONS;
use texlab::scenario::Scenario;

pub async fn run(
    scenario: &'static str,
    file: &'static str,
    position: Position,
) -> Option<HoverContents> {
    let scenario = format!("hover/{}", scenario);
    let scenario = Scenario::new(&scenario).await;
    scenario.open(file).await;
    let identifier = TextDocumentIdentifier::new(scenario.uri(file));
    let params = TextDocumentPositionParams::new(identifier, position);
    scenario
        .server
        .hover(params)
        .await
        .unwrap()
        .map(|hover| hover.contents)
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_entry_type_known() {
    let contents = run("bibtex/entry_type", "foo.bib", Position::new(0, 5))
        .await
        .unwrap();
    assert_eq!(
        contents,
        HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: Cow::from(LANGUAGE_OPTIONS.get_entry_type_doc("article").unwrap())
        })
    );
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_entry_type_unknown() {
    let contents = run("bibtex/entry_type", "foo.bib", Position::new(2, 2)).await;
    assert_eq!(contents, None);
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_field_known() {
    let contents = run("bibtex/field", "foo.bib", Position::new(1, 4))
        .await
        .unwrap();
    assert_eq!(
        contents,
        HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: Cow::from(LANGUAGE_OPTIONS.get_field_doc("author").unwrap())
        })
    )
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_field_unknown() {
    let contents = run("bibtex/field", "foo.bib", Position::new(2, 5)).await;
    assert_eq!(contents, None);
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_citation_latex() {
    let contents = run("latex/citation", "foo.tex", Position::new(2, 7)).await;
    assert_ne!(contents, None);
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_citation_bibtex() {
    let contents = run("latex/citation", "foo.bib", Position::new(0, 11)).await;
    assert_ne!(contents, None);
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_component_class() {
    let contents = run("latex/component", "foo.tex", Position::new(0, 19)).await;
    assert!(contents.is_some());
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_component_package() {
    let contents = run("latex/component", "foo.tex", Position::new(2, 16)).await;
    assert!(contents.is_some());
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_component_package_unknown() {
    let contents = run("latex/component", "foo.tex", Position::new(3, 14)).await;
    assert_eq!(contents, None);
}
