use insta::{
    assert_json_snapshot,
    internals::{Content, ContentPath},
};
use lsp_types::Url;

use crate::features::{goto_definition, testing::FeatureTester};

#[test]
fn test_empty_latex_document() {
    let links = goto_definition(
        FeatureTester::builder()
            .files(vec![("main.tex", "")])
            .main("main.tex")
            .line(0)
            .character(0)
            .build()
            .definition(),
    );

    assert_eq!(links, None);
}

#[test]
fn test_empty_bibtex_document() {
    let links = goto_definition(
        FeatureTester::builder()
            .files(vec![("main.bib", "")])
            .main("main.bib")
            .line(0)
            .character(0)
            .build()
            .definition(),
    );

    assert_eq!(links, None);
}

#[test]
fn test_command_definition() {
    let links = goto_definition(
        FeatureTester::builder()
            .files(vec![(
                "main.tex",
                r#"
\DeclareMathOperator{\foo}{foo}
\foo"#,
            )])
            .main("main.tex")
            .line(1)
            .character(2)
            .build()
            .definition(),
    );

    assert_json_snapshot!( links, { "[].targetUri" => insta::dynamic_redaction(redact_uri) });
}

#[test]
fn test_document_simple() {
    let links = goto_definition(
        FeatureTester::builder()
            .files(vec![
                ("foo.tex", r#"\addbibresource{baz.bib}"#),
                ("bar.bib", r#"@article{foo, bar = {baz}}"#),
                ("baz.bib", r#"@article{foo, bar = {baz}}"#),
            ])
            .main("foo.tex")
            .line(0)
            .character(18)
            .build()
            .definition(),
    );

    assert_json_snapshot!( links, { "[].targetUri" => insta::dynamic_redaction(redact_uri) });
}

#[test]
fn test_entry_simple() {
    let links = goto_definition(
        FeatureTester::builder()
            .files(vec![
                (
                    "foo.tex",
                    r#"
\addbibresource{baz.bib}
\cite{foo}"#,
                ),
                ("bar.bib", r#"@article{foo, bar = {baz}}"#),
                ("baz.bib", r#"@article{foo, bar = {baz}}"#),
            ])
            .main("foo.tex")
            .line(1)
            .character(6)
            .build()
            .definition(),
    );

    assert_json_snapshot!( links, { "[].targetUri" => insta::dynamic_redaction(redact_uri) });
}

#[test]
fn test_string_simple() {
    let links = goto_definition(
        FeatureTester::builder()
            .files(vec![(
                "main.bib",
                r#"
@string{foo = {bar}}
@article{bar, author = foo}"#,
            )])
            .main("main.bib")
            .line(1)
            .character(24)
            .build()
            .definition(),
    );

    assert_json_snapshot!( links, { "[].targetUri" => insta::dynamic_redaction(redact_uri) });
}

#[test]
fn test_string_concat() {
    let links = goto_definition(
        FeatureTester::builder()
            .files(vec![(
                "main.bib",
                r#"
@string{foo = {bar}}
@article{bar, author = foo # "bar"}"#,
            )])
            .main("main.bib")
            .line(1)
            .character(24)
            .build()
            .definition(),
    );

    assert_json_snapshot!( links, { "[].targetUri" => insta::dynamic_redaction(redact_uri) });
}

#[test]
fn test_string_field() {
    let links = goto_definition(
        FeatureTester::builder()
            .files(vec![(
                "main.bib",
                r#"
@string{foo = {bar}}
@article{bar, author = foo}"#,
            )])
            .main("main.bib")
            .line(1)
            .character(18)
            .build()
            .definition(),
    );

    assert_eq!(links, None);
}

fn redact_uri(value: Content, _path: ContentPath) -> String {
    value.as_str().unwrap().replace(
        Url::from_directory_path(std::env::temp_dir())
            .unwrap()
            .as_str(),
        "[tmp]/",
    )
}
