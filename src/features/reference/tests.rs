use insta::assert_json_snapshot;

use crate::features::{find_all_references, redactions::redact_uri, testing::FeatureTester};

#[test]
fn test_empty_latex_document() {
    let references = find_all_references(
        FeatureTester::builder()
            .files(vec![("main.tex", "")])
            .main("main.tex")
            .line(0)
            .character(0)
            .build()
            .reference(),
    );

    assert_eq!(references, Vec::new());
}

#[test]
fn test_empty_bibtex_document() {
    let references = find_all_references(
        FeatureTester::builder()
            .files(vec![("main.bib", "")])
            .main("main.bib")
            .line(0)
            .character(0)
            .build()
            .reference(),
    );

    assert_eq!(references, Vec::new());
}

#[test]
fn test_entry_definition() {
    let references = find_all_references(
        FeatureTester::builder()
            .files(vec![
                ("foo.bib", r#"@article{foo,}"#),
                ("bar.tex", r#"\cite{foo}\addbibresource{foo.bib}"#),
            ])
            .main("foo.bib")
            .line(0)
            .character(11)
            .build()
            .reference(),
    );

    assert_json_snapshot!(
        references,
        {
            "[].targetUri" => insta::dynamic_redaction(redact_uri),
            "." => insta::sorted_redaction(),
        }
    );
}

#[test]
fn test_entry_definition_include_declaration() {
    let references = find_all_references(
        FeatureTester::builder()
            .files(vec![
                ("foo.bib", r#"@article{foo,}"#),
                ("bar.tex", r#"\cite{foo}\addbibresource{foo.bib}"#),
            ])
            .main("foo.bib")
            .line(0)
            .character(11)
            .include_declaration(true)
            .build()
            .reference(),
    );

    assert_json_snapshot!(
        references,
        {
            "[].targetUri" => insta::dynamic_redaction(redact_uri),
            "." => insta::sorted_redaction(),
        }
    );
}

#[test]
fn test_entry_reference() {
    let references = find_all_references(
        FeatureTester::builder()
            .files(vec![
                ("foo.bib", r#"@article{foo,}"#),
                ("bar.tex", r#"\cite{foo}\addbibresource{foo.bib}"#),
            ])
            .main("bar.tex")
            .line(0)
            .character(8)
            .build()
            .reference(),
    );

    assert_json_snapshot!(
        references,
        {
            "[].targetUri" => insta::dynamic_redaction(redact_uri),
            "." => insta::sorted_redaction(),
        }
    );
}

#[test]
fn test_entry_reference_include_declaration() {
    let references = find_all_references(
        FeatureTester::builder()
            .files(vec![
                ("foo.bib", r#"@article{foo,}"#),
                ("bar.tex", r#"\cite{foo}\addbibresource{foo.bib}"#),
            ])
            .main("bar.tex")
            .line(0)
            .character(6)
            .include_declaration(true)
            .build()
            .reference(),
    );

    assert_json_snapshot!(
        references,
        {
            "[].targetUri" => insta::dynamic_redaction(redact_uri),
            "." => insta::sorted_redaction(),
        }
    );
}

#[test]
fn test_label_definition() {
    let references = find_all_references(
        FeatureTester::builder()
            .files(vec![
                ("foo.tex", r#"\label{foo}"#),
                ("bar.tex", r#"\ref{foo}\input{foo.tex}"#),
            ])
            .main("foo.tex")
            .line(0)
            .character(8)
            .build()
            .reference(),
    );

    assert_json_snapshot!(
        references,
        {
            "[].targetUri" => insta::dynamic_redaction(redact_uri),
            "." => insta::sorted_redaction(),
        }
    );
}

#[test]
fn test_label_definition_include_declaration() {
    let references = find_all_references(
        FeatureTester::builder()
            .files(vec![
                ("foo.tex", r#"\label{foo}\input{bar.tex}"#),
                ("bar.tex", r#"\ref{foo}"#),
            ])
            .main("foo.tex")
            .line(0)
            .character(9)
            .include_declaration(true)
            .build()
            .reference(),
    );

    assert_json_snapshot!(
        references,
        {
            "[].targetUri" => insta::dynamic_redaction(redact_uri),
            "." => insta::sorted_redaction(),
        }
    );
}

#[test]
fn test_label_reference() {
    let references = find_all_references(
        FeatureTester::builder()
            .files(vec![
                ("foo.tex", r#"\label{foo}\input{bar.tex}"#),
                ("bar.tex", r#"\ref{foo}"#),
                ("baz.tex", r#"\ref{foo}\input{bar.tex}"#),
            ])
            .main("bar.tex")
            .line(0)
            .character(7)
            .build()
            .reference(),
    );

    assert_json_snapshot!(
        references,
        {
            "[].targetUri" => insta::dynamic_redaction(redact_uri),
            "." => insta::sorted_redaction(),
        }
    );
}

#[test]
fn test_label_reference_include_declaration() {
    let references = find_all_references(
        FeatureTester::builder()
            .files(vec![
                ("foo.tex", r#"\label{foo}"#),
                ("bar.tex", r#"\ref{foo}\input{foo.tex}"#),
            ])
            .main("bar.tex")
            .line(0)
            .character(7)
            .include_declaration(true)
            .build()
            .reference(),
    );

    assert_json_snapshot!(
        references,
        {
            "[].targetUri" => insta::dynamic_redaction(redact_uri),
            "." => insta::sorted_redaction(),
        }
    );
}

#[test]
fn test_string_definition() {
    let references = find_all_references(
        FeatureTester::builder()
            .files(vec![(
                "main.bib",
                r#"
@string{foo = {Foo}}
@string{bar = {Bar}}
@article{baz, author = foo}"#,
            )])
            .main("main.bib")
            .line(2)
            .character(24)
            .build()
            .reference(),
    );

    assert_json_snapshot!(
        references,
        {
            "[].targetUri" => insta::dynamic_redaction(redact_uri),
            "." => insta::sorted_redaction(),
        }
    );
}

#[test]
fn test_string_definition_include_declaration() {
    let references = find_all_references(
        FeatureTester::builder()
            .files(vec![(
                "main.bib",
                r#"
@string{foo = {Foo}}
@string{bar = {Bar}}
@article{baz, author = foo}"#,
            )])
            .main("main.bib")
            .line(2)
            .character(24)
            .include_declaration(true)
            .build()
            .reference(),
    );

    assert_json_snapshot!(
        references,
        {
            "[].targetUri" => insta::dynamic_redaction(redact_uri),
            "." => insta::sorted_redaction(),
        }
    );
}

#[test]
fn test_string_reference() {
    let references = find_all_references(
        FeatureTester::builder()
            .files(vec![(
                "main.bib",
                r#"
@string{foo = {Foo}}
@string{bar = {Bar}}
@article{baz, author = foo}"#,
            )])
            .main("main.bib")
            .line(0)
            .character(10)
            .build()
            .reference(),
    );

    assert_json_snapshot!(
        references,
        {
            "[].targetUri" => insta::dynamic_redaction(redact_uri),
            "." => insta::sorted_redaction(),
        }
    );
}

#[test]
fn test_string_reference_include_declaration() {
    let references = find_all_references(
        FeatureTester::builder()
            .files(vec![(
                "main.bib",
                r#"
@string{foo = {Foo}}
@string{bar = {Bar}}
@article{baz, author = foo}"#,
            )])
            .main("main.bib")
            .line(0)
            .character(10)
            .include_declaration(true)
            .build()
            .reference(),
    );

    assert_json_snapshot!(
        references,
        {
            "[].targetUri" => insta::dynamic_redaction(redact_uri),
            "." => insta::sorted_redaction(),
        }
    );
}
