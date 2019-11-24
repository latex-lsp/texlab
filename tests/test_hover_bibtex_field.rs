pub mod support;

use lsp_types::*;
use support::hover::*;
use texlab::syntax::LANGUAGE_DATA;

const SCENARIO: &str = "bibtex/field";

#[tokio::test]
async fn known() {
    let contents = run(SCENARIO, "foo.bib", 1, 4).await.unwrap();
    assert_eq!(
        contents,
        HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: LANGUAGE_DATA
                .field_documentation("author")
                .unwrap()
                .to_owned()
        })
    );
}

#[tokio::test]
async fn unknown() {
    let contents = run(SCENARIO, "foo.bib", 2, 5).await;
    assert_eq!(contents, None);
}
