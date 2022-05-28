use insta::assert_json_snapshot;

use crate::features::{find_hover, testing::FeatureTester};

#[test]
fn test_empty_latex_document() {
    let hover = find_hover(
        FeatureTester::builder()
            .files(vec![("main.tex", "")])
            .main("main.tex")
            .line(0)
            .character(0)
            .build()
            .hover(),
    );

    assert_eq!(hover, None);
}

#[test]
fn test_empty_bibtex_document() {
    let hover = find_hover(
        FeatureTester::builder()
            .files(vec![("main.bib", "")])
            .main("main.bib")
            .line(0)
            .character(0)
            .build()
            .hover(),
    );

    assert_eq!(hover, None);
}

#[test]
fn test_citation_inside_cite() {
    let hover = find_hover(
        FeatureTester::builder()
            .files(vec![
                (
                    "main.bib",
                    "@article{foo, author = {Foo Bar}, title = {Baz Qux}, year = 1337}",
                ),
                (
                    "main.tex",
                    r#"
\addbibresource{main.bib}
\cite{foo}"#,
                ),
            ])
            .main("main.tex")
            .line(1)
            .character(7)
            .build()
            .hover(),
    );

    assert_json_snapshot!(hover);
}

#[test]
fn test_citation_inside_entry() {
    let hover = find_hover(
        FeatureTester::builder()
            .files(vec![
                (
                    "main.bib",
                    "@article{foo, author = {Foo Bar}, title = {Baz Qux}, year = 1337}",
                ),
                (
                    "main.tex",
                    r#"
\addbibresource{main.bib}
\cite{foo}"#,
                ),
            ])
            .main("main.bib")
            .line(0)
            .character(11)
            .build()
            .hover(),
    );

    assert_json_snapshot!(hover);
}

#[test]
fn test_component_known_package() {
    let hover = find_hover(
        FeatureTester::builder()
            .files(vec![("main.tex", r#"\usepackage{amsmath}"#)])
            .main("main.tex")
            .line(0)
            .character(15)
            .build()
            .hover(),
    );

    assert_json_snapshot!(hover);
}

#[test]
fn test_component_unknown_class() {
    let hover = find_hover(
        FeatureTester::builder()
            .files(vec![("main.tex", r#"\documentclass{abcdefghijklmnop}"#)])
            .main("main.tex")
            .line(0)
            .character(20)
            .build()
            .hover(),
    );

    assert_eq!(hover, None);
}

#[test]
fn test_entry_type_known_type() {
    let hover = find_hover(
        FeatureTester::builder()
            .files(vec![("main.bib", r#"@article{foo,}"#)])
            .main("main.bib")
            .line(0)
            .character(3)
            .build()
            .hover(),
    );

    assert_json_snapshot!(hover);
}

#[test]
fn test_entry_type_unknown_field() {
    let hover = find_hover(
        FeatureTester::builder()
            .files(vec![("main.bib", r#"@foo{bar,}"#)])
            .main("main.bib")
            .line(0)
            .character(3)
            .build()
            .hover(),
    );

    assert_eq!(hover, None);
}

#[test]
fn test_entry_type_key() {
    let hover = find_hover(
        FeatureTester::builder()
            .files(vec![("main.bib", r#"@article{foo,}"#)])
            .main("main.bib")
            .line(0)
            .character(11)
            .build()
            .hover(),
    );

    assert_eq!(hover, None);
}

#[test]
fn test_field_known() {
    let hover = find_hover(
        FeatureTester::builder()
            .files(vec![("main.bib", r#"@article{foo, author = bar}"#)])
            .main("main.bib")
            .line(0)
            .character(15)
            .build()
            .hover(),
    );

    assert_json_snapshot!(hover);
}

#[test]
fn test_field_unknown() {
    let hover = find_hover(
        FeatureTester::builder()
            .files(vec![("main.bib", r#"@article{foo, bar = baz}"#)])
            .main("main.bib")
            .line(0)
            .character(15)
            .build()
            .hover(),
    );

    assert_eq!(hover, None);
}

#[test]
fn test_field_entry_key() {
    let hover = find_hover(
        FeatureTester::builder()
            .files(vec![("main.bib", r#"@article{foo, author = bar}"#)])
            .main("main.bib")
            .line(0)
            .character(11)
            .build()
            .hover(),
    );

    assert_eq!(hover, None);
}

#[test]
fn test_section() {
    let hover = find_hover(
        FeatureTester::builder()
            .files(vec![("main.tex", r#"\section{Foo}\label{sec:foo}"#)])
            .main("main.tex")
            .line(0)
            .character(23)
            .build()
            .hover(),
    );

    assert_json_snapshot!(hover);
}

#[test]
fn test_string_inside_reference() {
    let hover = find_hover(
        FeatureTester::builder()
            .files(vec![(
                "main.bib",
                r#"
@string{foo = "Foo"}
@string{bar = "Bar"}
@article{baz, author = bar}"#,
            )])
            .main("main.bib")
            .line(2)
            .character(24)
            .build()
            .hover(),
    );

    assert_json_snapshot!(hover);
}

#[test]
fn test_string_inside_field() {
    let hover = find_hover(
        FeatureTester::builder()
            .files(vec![(
                "main.bib",
                r#"
@string{foo = "Foo"}
@string{bar = "Bar"}
@article{baz, author = bar}"#,
            )])
            .main("main.bib")
            .line(2)
            .character(21)
            .build()
            .hover(),
    );

    assert_eq!(hover, None);
}
