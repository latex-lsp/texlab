use texlab_protocol::*;
use texlab_syntax::LANGUAGE_DATA;
use texlab_test::hover::*;

const SCENARIO: &str = "bibtex/type";

#[tokio::test]
async fn known() {
    let contents = run(SCENARIO, "foo.bib", 0, 5).await.unwrap();
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
async fn unknown() {
    let contents = run(SCENARIO, "foo.bib", 2, 2).await;
    assert_eq!(contents, None);
}
