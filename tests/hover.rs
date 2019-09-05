use lsp_types::*;
use texlab::scenario::{Scenario, FULL_CAPABILITIES};
use texlab::syntax::LANGUAGE_DATA;

pub async fn run(
    scenario: &'static str,
    file: &'static str,
    position: Position,
) -> Option<HoverContents> {
    let scenario = format!("hover/{}", scenario);
    let scenario = Scenario::new(&scenario, &FULL_CAPABILITIES).await;
    scenario.open(file).await;
    let identifier = TextDocumentIdentifier::new(scenario.uri(file).into());
    let params = TextDocumentPositionParams::new(identifier, position);
    let contents = scenario
        .server
        .hover(params)
        .await
        .unwrap()
        .map(|hover| hover.contents);
    contents
}

#[tokio::test]
async fn test_entry_type_known() {
    let contents = run("bibtex/entry_type", "foo.bib", Position::new(0, 5))
        .await
        .unwrap();
    assert_eq!(
        contents,
        HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: LANGUAGE_DATA
                .entry_type_documentation("article")
                .unwrap()
                .to_owned()
        })
    );
}

#[tokio::test]
async fn test_entry_type_unknown() {
    let contents = run("bibtex/entry_type", "foo.bib", Position::new(2, 2)).await;
    assert_eq!(contents, None);
}

#[tokio::test]
async fn test_field_known() {
    let contents = run("bibtex/field", "foo.bib", Position::new(1, 4))
        .await
        .unwrap();
    assert_eq!(
        contents,
        HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: LANGUAGE_DATA
                .field_documentation("author")
                .unwrap()
                .to_owned()
        })
    )
}

#[tokio::test]
async fn test_field_unknown() {
    let contents = run("bibtex/field", "foo.bib", Position::new(2, 5)).await;
    assert_eq!(contents, None);
}

#[tokio::test]
async fn test_citation_latex() {
    let contents = run("latex/citation", "foo.tex", Position::new(2, 7)).await;
    assert_ne!(contents, None);
}

#[tokio::test]
async fn test_citation_bibtex() {
    let contents = run("latex/citation", "foo.bib", Position::new(0, 11)).await;
    assert_ne!(contents, None);
}

#[tokio::test]
async fn test_component_class() {
    let contents = run("latex/component", "foo.tex", Position::new(0, 19)).await;
    assert!(contents.is_some());
}

#[tokio::test]
async fn test_component_package() {
    let contents = run("latex/component", "foo.tex", Position::new(2, 16)).await;
    assert!(contents.is_some());
}

#[tokio::test]
async fn test_component_package_unknown() {
    let contents = run("latex/component", "foo.tex", Position::new(3, 14)).await;
    assert_eq!(contents, None);
}
