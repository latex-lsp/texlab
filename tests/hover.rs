#![feature(await_macro, async_await)]

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
    let scenario = await!(Scenario::new(&scenario));
    await!(scenario.open(file));
    let identifier = TextDocumentIdentifier::new(scenario.uri(file));
    let params = TextDocumentPositionParams::new(identifier, position);
    await!(scenario.server.hover(params))
        .unwrap()
        .map(|hover| hover.contents)
}

#[test]
fn test_entry_type_known() {
    block_on(async move {
        let contents = await!(run("bibtex/entry_type", "foo.bib", Position::new(0, 5))).unwrap();
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
        let contents = await!(run("bibtex/entry_type", "foo.bib", Position::new(2, 2)));
        assert_eq!(contents, None);
    });
}

#[test]
fn test_field_known() {
    block_on(async move {
        let contents = await!(run("bibtex/field", "foo.bib", Position::new(1, 4))).unwrap();
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
        let contents = await!(run("bibtex/field", "foo.bib", Position::new(2, 5)));
        assert_eq!(contents, None);
    });
}

#[test]
fn test_citation_bibtex() {
    block_on(async move {
        let contents = await!(run("latex/citation", "foo.bib", Position::new(0, 10)));
        assert_eq!(contents.is_some(), true);
    });
}

#[test]
fn test_citation_latex() {
    block_on(async move {
        let contents = await!(run("latex/citation", "foo.tex", Position::new(2, 8)));
        assert_eq!(contents.is_some(), true);
    });
}

#[test]
fn test_component_class() {
    block_on(async move {
        let contents = await!(run("latex/component", "foo.tex", Position::new(0, 19)));
        assert_eq!(contents.is_some(), true);
    });
}

#[test]
fn test_component_package() {
    block_on(async move {
        let contents = await!(run("latex/component", "foo.tex", Position::new(2, 16)));
        assert_eq!(contents.is_some(), true);
    });
}

#[test]
fn test_component_package_unknown() {
    block_on(async move {
        let contents = await!(run("latex/component", "foo.tex", Position::new(3, 14)));
        assert_eq!(contents, None);
    });
}
