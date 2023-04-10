use insta::assert_json_snapshot;
use lsp_types::{request::HoverRequest, ClientCapabilities, HoverContents, HoverParams};

use crate::fixture::TestBed;

fn find_hover(fixture: &str) -> Option<HoverContents> {
    let test_bed = TestBed::new(fixture).unwrap();

    test_bed.initialize(ClientCapabilities::default()).unwrap();

    let text_document_position_params = test_bed.cursor().unwrap();

    test_bed
        .client()
        .send_request::<HoverRequest>(HoverParams {
            text_document_position_params,
            work_done_progress_params: Default::default(),
        })
        .unwrap()
        .map(|hover| {
            assert_eq!(hover.range, Some(test_bed.locations()[0].range));
            hover.contents
        })
}

#[test]
fn empty_latex_document() {
    assert_eq!(
        find_hover(
            r#"
%! main.tex

|"#
        ),
        None,
    );
}

#[test]
fn empty_bibtex_document() {
    assert_eq!(
        find_hover(
            r#"
%! main.bib

|"#
        ),
        None,
    );
}

#[test]
fn citation_inside_cite() {
    assert_json_snapshot!(find_hover(
        r#"
%! main.bib
@article{foo, author = {Foo Bar}, title = {Baz Qux}, year = 1337}

%! main.tex
\addbibresource{main.bib}
\cite{foo}
       |
      ^^^"#
    ));
}

#[test]
fn citation_inside_entry() {
    assert_json_snapshot!(find_hover(
        r#"
%! main.bib
@article{foo, author = {Foo Bar}, title = {Baz Qux}, year = 1337}
          |
         ^^^

%! main.tex
\addbibresource{main.bib}
\cite{foo}"#
    ));
}

#[test]
fn component_known_package() {
    assert_json_snapshot!(find_hover(
        r#"
%! main.tex
\usepackage{amsmath}
             |
            ^^^^^^^"#
    ));
}

#[test]
fn component_unknown_class() {
    assert_eq!(
        find_hover(
            r#"
%! main.tex
\documentclass{abcdefghijklmnop}
                    |"#
        ),
        None,
    );
}

#[test]
fn entry_type_known_type() {
    assert_json_snapshot!(find_hover(
        r#"
%! main.bib
@article{foo,}
    |
^^^^^^^^"#
    ));
}

#[test]
fn entry_type_unknown_field() {
    assert_eq!(
        find_hover(
            r#"
%! main.bib
@foo{bar,}
  |"#
        ),
        None,
    );
}

#[test]
fn entry_type_key() {
    assert_eq!(
        find_hover(
            r#"
%! main.bib
@foo{bar,}
      |"#
        ),
        None,
    );
}

#[test]
fn field_known() {
    assert_json_snapshot!(find_hover(
        r#"
%! main.bib
@article{foo, author = bar}
               |
              ^^^^^^"#
    ));
}

#[test]
fn field_unknown() {
    assert_eq!(
        find_hover(
            r#"
%! main.bib
@article{foo, bar = baz}
               |"#
        ),
        None,
    );
}

#[test]
fn section() {
    assert_json_snapshot!(find_hover(
        r#"
%! main.tex
\section{Foo}
\label{sec:foo}
         |
       ^^^^^^^"#,
    ));
}

#[test]
fn string_inside_reference() {
    assert_json_snapshot!(find_hover(
        r#"
%! main.bib
@string{foo = "Foo"}
@string{bar = "Bar"}
@article{baz, author = bar}
                        |
                       ^^^"#
    ));
}

#[test]
fn string_inside_field() {
    assert_eq!(
        find_hover(
            r#"
%! main.bib
@string{foo = "Foo"}
@string{bar = "Bar"}
@article{baz, author = bar}
                     |"#
        ),
        None,
    );
}

#[test]
fn label_theorem_child_file() {
    assert_json_snapshot!(find_hover(
        r#"
%! main.tex
\documentclass{article}
\newtheorem{lemma}{Lemma}
\include{child}
\ref{thm:foo}
        |
     ^^^^^^^

%! child.tex
\begin{lemma}\label{thm:foo}
    1 + 1 = 2
\end{lemma}"#
    ));
}

#[test]
fn label_theorem_child_file_mumber() {
    assert_json_snapshot!(find_hover(
        r#"
%! main.tex
\documentclass{article}
\newtheorem{lemma}{Lemma}
\include{child}
\ref{thm:foo}
        |
     ^^^^^^^

%! child.tex
\begin{lemma}[Foo]\label{thm:foo}
    1 + 1 = 2
\end{lemma}

%! child.aux
\newlabel{thm:foo}{{1}{1}{Foo}{lemma.1}{}}"#
    ));
}
