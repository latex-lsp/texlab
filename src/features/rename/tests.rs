use insta::assert_json_snapshot;

use crate::features::{redactions::redact_uri, testing::FeatureTester};

use super::rename_all;

#[test]
fn test_command() {
    let edit = rename_all(
        FeatureTester::builder()
            .files(vec![
                ("foo.tex", r#"\baz\include{bar.tex}"#),
                ("bar.tex", r#"\baz"#),
            ])
            .main("foo.tex")
            .line(0)
            .character(2)
            .new_name("qux")
            .build()
            .rename(),
    );

    assert_json_snapshot!(
        edit,
        {
            ".changes.$key" => insta::dynamic_redaction(redact_uri),
            ".changes" => insta::sorted_redaction(),
        }
    );
}

#[test]
fn test_entry() {
    let edit = rename_all(
        FeatureTester::builder()
            .files(vec![
                ("main.bib", r#"@article{foo, bar = baz}"#),
                ("main.tex", "\\addbibresource{main.bib}\n\\cite{foo}"),
            ])
            .main("main.bib")
            .line(0)
            .character(9)
            .new_name("qux")
            .build()
            .rename(),
    );

    assert_json_snapshot!(
        edit,
        {
            ".changes.$key" => insta::dynamic_redaction(redact_uri),
            ".changes" => insta::sorted_redaction(),
        }
    );
}

#[test]
fn test_citation() {
    let edit = rename_all(
        FeatureTester::builder()
            .files(vec![
                ("main.bib", r#"@article{foo, bar = baz}"#),
                ("main.tex", "\\addbibresource{main.bib}\n\\cite{foo}"),
            ])
            .main("main.tex")
            .line(1)
            .character(6)
            .new_name("qux")
            .build()
            .rename(),
    );

    assert_json_snapshot!(
        edit,
        {
            ".changes.$key" => insta::dynamic_redaction(redact_uri),
            ".changes" => insta::sorted_redaction(),
        }
    );
}

#[test]
fn test_label() {
    let edit = rename_all(
        FeatureTester::builder()
            .files(vec![
                ("foo.tex", r#"\label{foo}\include{bar}"#),
                ("bar.tex", r#"\ref{foo}"#),
                ("baz.tex", r#"\ref{foo}"#),
            ])
            .main("foo.tex")
            .line(0)
            .character(7)
            .new_name("bar")
            .build()
            .rename(),
    );

    assert_json_snapshot!(
        edit,
        {
            ".changes.$key" => insta::dynamic_redaction(redact_uri),
            ".changes" => insta::sorted_redaction(),
        }
    );
}
