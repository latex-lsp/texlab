use insta::assert_json_snapshot;

use crate::features::{find_document_links, redactions::redact_uri, testing::FeatureTester};

#[test]
fn test_empty_latex_document() {
    let links = find_document_links(
        FeatureTester::builder()
            .files(vec![("main.tex", "")])
            .main("main.tex")
            .build()
            .link(),
    );

    assert_eq!(links, Vec::new());
}

#[test]
fn test_empty_bibtex_document() {
    let links = find_document_links(
        FeatureTester::builder()
            .files(vec![("main.bib", "")])
            .main("main.bib")
            .build()
            .link(),
    );

    assert_eq!(links, Vec::new());
}

#[test]
fn test_includes() {
    let links = find_document_links(
        FeatureTester::builder()
            .files(vec![("foo.tex", r#"\input{bar.tex}"#), ("bar.tex", r#""#)])
            .main("foo.tex")
            .build()
            .link(),
    );

    assert_json_snapshot!(
        links,
        {
            "[].target" => insta::dynamic_redaction(redact_uri),
            "." => insta::sorted_redaction(),
        }
    );
}

#[test]
fn test_imports() {
    let links = find_document_links(
        FeatureTester::builder()
            .files(vec![
                ("foo.tex", r#"\import{bar/}{baz}"#),
                ("bar/baz.tex", r#""#),
            ])
            .main("foo.tex")
            .build()
            .link(),
    );

    assert_json_snapshot!(
        links,
        {
            "[].target" => insta::dynamic_redaction(redact_uri),
            "." => insta::sorted_redaction(),
        }
    );
}
