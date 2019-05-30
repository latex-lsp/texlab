#![feature(async_await)]

use futures::executor::block_on;
use lsp_types::*;
use std::borrow::Cow;
use texlab::data::bibtex_entry_type;
use texlab::data::bibtex_field;
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

#[test]
fn test_entry_type_known() {
    block_on(async move {
        let contents = run("bibtex/entry_type", "foo.bib", Position::new(0, 5))
            .await
            .unwrap();
        assert_eq!(
            contents,
            HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: Cow::from(bibtex_entry_type::get_documentation("article").unwrap())
            })
        );
    });
}

#[test]
fn test_entry_type_unknown() {
    block_on(async move {
        let contents = run("bibtex/entry_type", "foo.bib", Position::new(2, 2)).await;
        assert_eq!(contents, None);
    });
}

#[test]
fn test_field_known() {
    block_on(async move {
        let contents = run("bibtex/field", "foo.bib", Position::new(1, 4))
            .await
            .unwrap();
        assert_eq!(
            contents,
            HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: Cow::from(bibtex_field::get_documentation("author").unwrap())
            })
        )
    });
}

#[test]
fn test_field_unknown() {
    block_on(async move {
        let contents = run("bibtex/field", "foo.bib", Position::new(2, 5)).await;
        assert_eq!(contents, None);
    });
}

#[test]
fn test_citation_bibtex() {
    block_on(async move {
        let contents = run("latex/citation", "foo.bib", Position::new(0, 10)).await;
        assert_eq!(contents.is_some(), true);
    });
}

#[test]
fn test_citation_latex() {
    block_on(async move {
        let contents = run("latex/citation", "foo.tex", Position::new(2, 8)).await;
        assert_eq!(contents.is_some(), true);
    });
}

#[test]
fn test_component_class() {
    block_on(async move {
        let contents = run("latex/component", "foo.tex", Position::new(0, 19)).await;
        assert_eq!(contents.is_some(), true);
    });
}

#[test]
fn test_component_package() {
    block_on(async move {
        let contents = run("latex/component", "foo.tex", Position::new(2, 16)).await;
        assert_eq!(contents.is_some(), true);
    });
}

#[test]
fn test_component_package_unknown() {
    block_on(async move {
        let contents = run("latex/component", "foo.tex", Position::new(3, 14)).await;
        assert_eq!(contents, None);
    });
}
