pub mod support;

use lsp_types::*;
use support::completion::*;

const SCENARIO: &str = "latex/citation";

#[tokio::test]
async fn valid_citation() {
    let item = run_item(SCENARIO, "foo.tex", 5, 6, "foo:2019").await;
    verify::text_edit(&item, 5, 6, 5, 6, "foo:2019");
    assert_eq!(
        item.documentation.unwrap(),
        Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "Bar, F. (2019). Baz Qux.".into()
        })
    );
}

#[tokio::test]
async fn invalid_citation() {
    let item = run_item(SCENARIO, "foo.tex", 5, 6, "bar:2005").await;
    verify::text_edit(&item, 5, 6, 5, 6, "bar:2005");
    assert_eq!(item.documentation, None);
}
