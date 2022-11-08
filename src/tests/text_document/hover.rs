use anyhow::Result;
use lsp_types::{
    request::HoverRequest, ClientCapabilities, Hover, HoverContents, HoverParams, MarkupContent,
    MarkupKind,
};

use crate::{
    tests::{client::Client, fixture},
    util::components::COMPONENT_DATABASE,
    LANGUAGE_DATA,
};

fn check(fixture: &str, contents: Option<HoverContents>) -> Result<()> {
    let mut client = Client::spawn()?;
    client.initialize(ClientCapabilities::default(), None)?;

    let fixture = fixture::parse(fixture);
    for file in fixture.files {
        client.open(file.name, file.lang, file.text)?;
    }

    let range = fixture
        .ranges
        .values()
        .next()
        .and_then(|map| map.values().next())
        .map(|file_range| file_range.range);

    let actual_hover = client.request::<HoverRequest>(HoverParams {
        text_document_position_params: fixture.cursor.unwrap().into_params(&client)?,
        work_done_progress_params: Default::default(),
    })?;

    client.shutdown()?;

    let expected_hover = contents.map(|contents| Hover { range, contents });

    assert_eq!(actual_hover, expected_hover);
    Ok(())
}

#[test]
fn empty_latex_document() -> Result<()> {
    check(
        r#"
%TEX main.tex
%SRC 
%CUR ^
"#,
        None,
    )
}

#[test]
fn empty_bibtex_document() -> Result<()> {
    check(
        r#"
%BIB main.bib
%SRC 
%CUR ^
"#,
        None,
    )
}

#[test]
fn citation_inside_cite() -> Result<()> {
    check(
        r#"
%BIB main.bib
%SRC @article{foo, author = {Foo Bar}, title = {Baz Qux}, year = 1337}

%TEX main.tex
%SRC \addbibresource{main.bib}
%SRC \cite{foo}
%CUR        ^
%1.1       ^^^
"#,
        Some(HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "F. Bar: \"Baz Qux\". (1337).".to_string(),
        })),
    )
}

#[test]
fn citation_inside_entry() -> Result<()> {
    check(
        r#"
%BIB main.bib
%SRC @article{foo, author = {Foo Bar}, title = {Baz Qux}, year = 1337}
%CUR           ^
%1.1          ^^^

%TEX main.tex
%SRC \addbibresource{main.bib}
%SRC \cite{foo}
"#,
        Some(HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "F. Bar: \"Baz Qux\". (1337).".to_string(),
        })),
    )
}

#[test]
fn component_known_package() -> Result<()> {
    check(
        r#"
%TEX main.tex
%SRC \usepackage{amsmath}
%CUR              ^
%1.1             ^^^^^^^
"#,
        COMPONENT_DATABASE
            .documentation("amsmath")
            .map(HoverContents::Markup),
    )
}

#[test]
fn component_unknown_class() -> Result<()> {
    check(
        r#"
%TEX main.tex
%SRC \documentclass{abcdefghijklmnop}
%CUR                     ^
"#,
        None,
    )
}

#[test]
fn entry_type_known_type() -> Result<()> {
    check(
        r#"
%BIB main.bib
%SRC @article{foo,}
%CUR     ^
%1.1 ^^^^^^^^
"#,
        Some(HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: LANGUAGE_DATA
                .entry_type_documentation("article")
                .unwrap()
                .to_string(),
        })),
    )
}

#[test]
fn entry_type_unknown_field() -> Result<()> {
    check(
        r#"
%BIB main.bib
%SRC @foo{bar,}
%CUR   ^
"#,
        None,
    )
}

#[test]
fn entry_type_key() -> Result<()> {
    check(
        r#"
%BIB main.bib
%SRC @foo{bar,}
%CUR       ^
"#,
        None,
    )
}

#[test]
fn field_known() -> Result<()> {
    check(
        r#"
%BIB main.bib
%SRC @article{foo, author = bar}
%CUR                ^
%1.1               ^^^^^^
"#,
        Some(HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: LANGUAGE_DATA
                .field_documentation("author")
                .unwrap()
                .to_string(),
        })),
    )
}

#[test]
fn field_unknown() -> Result<()> {
    check(
        r#"
%BIB main.bib
%SRC @article{foo, bar = baz}
%CUR                ^
"#,
        None,
    )
}

#[test]
fn section() -> Result<()> {
    check(
        r#"
%TEX main.tex
%SRC \section{Foo}
%SRC \label{sec:foo}
%CUR          ^
%1.1        ^^^^^^^
"#,
        Some(HoverContents::Markup(MarkupContent {
            kind: MarkupKind::PlainText,
            value: "Section (Foo)".to_string(),
        })),
    )
}

#[test]
fn string_inside_reference() -> Result<()> {
    check(
        r#"
%BIB main.bib
%SRC @string{foo = "Foo"}
%SRC @string{bar = "Bar"}
%SRC @article{baz, author = bar}
%CUR                         ^
%1.1                        ^^^
"#,
        Some(HoverContents::Markup(MarkupContent {
            kind: MarkupKind::PlainText,
            value: "Bar".to_string(),
        })),
    )
}

#[test]
fn string_inside_field() -> Result<()> {
    check(
        r#"
%BIB main.bib
%SRC @string{foo = "Foo"}
%SRC @string{bar = "Bar"}
%SRC @article{baz, author = bar}
%CUR                      ^
"#,
        None,
    )
}

#[test]
fn label_theorem_child_file() -> Result<()> {
    check(
        r#"
%TEX main.tex
%SRC \documentclass{article}
%SRC \newtheorem{lemma}{Lemma}
%SRC \include{child}
%SRC \ref{thm:foo}
%CUR         ^
%1.1      ^^^^^^^

%TEX child.tex
%SRC \begin{lemma}\label{thm:foo}
%SRC     1 + 1 = 2
%SRC \end{lemma}
"#,
        Some(HoverContents::Markup(MarkupContent {
            kind: MarkupKind::PlainText,
            value: "Lemma".to_string(),
        })),
    )
}

#[test]
fn label_theorem_child_file_mumber() -> Result<()> {
    check(
        r#"
%TEX main.tex
%SRC \documentclass{article}
%SRC \newtheorem{lemma}{Lemma}
%SRC \include{child}
%SRC \ref{thm:foo}
%CUR         ^
%1.1      ^^^^^^^

%TEX child.tex
%SRC \begin{lemma}[Foo]\label{thm:foo}
%SRC     1 + 1 = 2
%SRC \end{lemma}

%TEX child.aux
%SRC \newlabel{thm:foo}{{1}{1}{Foo}{lemma.1}{}}
"#,
        Some(HoverContents::Markup(MarkupContent {
            kind: MarkupKind::PlainText,
            value: "Lemma 1 (Foo)".to_string(),
        })),
    )
}
