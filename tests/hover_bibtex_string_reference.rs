use texlab_protocol::*;
use texlab_test::hover::*;

const SCENARIO: &str = "bibtex/string_reference";

#[tokio::test]
async fn valid() {
    let contents = run(SCENARIO, "foo.bib", 3, 15).await.unwrap();
    assert_eq!(
        contents,
        HoverContents::Markup(MarkupContent {
            kind: MarkupKind::PlainText,
            value: "\"foo {bar} baz\"".into(),
        })
    );
}

#[tokio::test]
async fn invalid() {
    let contents = run(SCENARIO, "foo.bib", 3, 20).await;
    assert_eq!(contents, None);
}
