use insta::assert_json_snapshot;
use itertools::Itertools;
use lsp_types::{
    request::{Completion, ResolveCompletionItem},
    CompletionItem, CompletionParams, CompletionResponse, CompletionTextEdit, Range,
};

use crate::fixture::TestBed;

fn complete(fixture: &str) -> Vec<CompletionItem> {
    let test_bed = TestBed::new(fixture).unwrap();
    test_bed
        .initialize(
            serde_json::from_value(serde_json::json!({
                "textDocument": {
                    "completion": {
                        "completionItem": {
                            "documentationFormat": ["plaintext", "markdown"]
                        }
                    }
                }
            }))
            .unwrap(),
        )
        .unwrap();

    let text_document_position = test_bed.cursor().unwrap();
    let position = text_document_position.position;
    let range = test_bed
        .locations()
        .first()
        .map_or_else(|| Range::new(position, position), |location| location.range);

    let items = match test_bed
        .client()
        .send_request::<Completion>(CompletionParams {
            text_document_position,
            partial_result_params: Default::default(),
            work_done_progress_params: Default::default(),
            context: None,
        })
        .unwrap()
    {
        Some(CompletionResponse::Array(items)) => items,
        Some(CompletionResponse::List(list)) => list.items,
        None => Vec::new(),
    };

    for item in &items {
        if let Some(CompletionTextEdit::Edit(edit)) = item.text_edit.as_ref() {
            assert_eq!(edit.range, range);
        }
    }

    items
        .into_iter()
        .take(5)
        .map(|item| {
            let mut item = test_bed
                .client()
                .send_request::<ResolveCompletionItem>(item)
                .unwrap();

            item.data = None;
            item.kind = None;
            item.sort_text = None;
            item.documentation = None;
            item.text_edit = None;
            item
        })
        .sorted_by(|item1, item2| item1.label.cmp(&item2.label))
        .collect()
}

#[test]
fn acronym_ref_simple() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
\acrshort{f}
          |
          ^"#
    ));
}

#[test]
fn acronym_ref_empty() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
\acrshort{}
          |"#
    ));
}

#[test]
fn acronym_ref_after_group() {
    assert_eq!(
        complete(
            r#"
%! main.tex
\newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
\acrshort{}
           |"#,
        ),
        Vec::new()
    );
}

#[test]
fn acronym_ref_open_brace() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
\acrshort{f
          |
          ^"#
    ));
}

#[test]
fn acronym_package_ref() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\acrodef{fpsLabel}[FPS]{Frames per Second}
\ac{f
    |
    ^"#
    ));
}

#[test]
fn glossary_ref_simple() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
\gls{f}
     |
     ^"#
    ));
}

#[test]
fn glossary_ref_open_brace() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
\gls{f
     |
     ^"#
    ));
}

#[test]
fn argument_empty() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\usepackage{amsfonts}
\mathbb{}
        |"#
    ));
}

#[test]
fn argument_word() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\usepackage{amsfonts}
\mathbb{A}
        |
        ^"#
    ));
}

#[test]
fn argument_open_brace() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\usepackage{amsfonts}
\mathbb{
        |
Test"#
    ));
}

#[test]
fn argument_open_brace_unrelated() {
    assert_eq!(
        complete(
            r#"
%! main.tex
\usepackage{amsfonts}
\mathbb{}{
          |
Test"#,
        ),
        Vec::new()
    );
}

#[test]
fn begin_environment_without_snippet_support() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\beg
    |
 ^^^"#
    ));
}

#[test]
fn citation() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\documentclass{article}
\bibliography{main}
\begin{document}
\cite{
      |
\end{document}

%! main.bib
@article{foo:2019,
    author = {Foo Bar},
    title = {Baz Qux},
    year = {2019},
}

@article{bar:2005,}"#
    ));
}

#[test]
fn citation_open_brace() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\addbibresource{main.bib}
\cite{
      |

%! main.bib
@article{foo,}"#
    ));
}

#[test]
fn citation_open_brace_multiple() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\addbibresource{main.bib}
\cite{foo,a
          |
          ^

%! main.bib
@article{foo,}"#
    ));
}

#[test]
fn citation_acronym() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\addbibresource{main.bib}
\DeclareAcronym{foo}{cite={}}
                           |

%! main.bib
@article{foo,}"#
    ));
}

#[test]
fn citation_after_brace() {
    assert_eq!(
        complete(
            r#"
%! main.tex
\documentclass{article}
\bibliography{main}
\begin{document}
\cite{}
       |
\end{document}

%! main.bib
@article{foo,}"#,
        ),
        Vec::new()
    );
}

#[test]
fn color_model_definition_simple() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\definecolor{foo}{}
                  |"#
    ));
}

#[test]
fn color_model_definition_open_brace() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\definecolor{foo}{
                  |"#
    ));
}

#[test]
fn color_model_definition_set_simple() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\definecolorset{}
                |"#
    ));
}

#[test]
fn color_model_definition_set_open_brace() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\definecolorset{
                |"#
    ));
}

#[test]
fn color_simple() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\color{}
       |"#
    ));
}

#[test]
fn color_word() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\color{re}
        |
       ^^"#
    ));
}

#[test]
fn color_open_brace() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\color{
       |"#
    ));
}

#[test]
fn component_command_simple() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\
 |"#
    ));
}

#[test]
fn component_command_simple_before() {
    assert_eq!(
        complete(
            r#"
%! main.tex
\
|"#,
        ),
        Vec::new()
    );
}

#[test]
fn component_command_simple_package() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\usepackage{lipsum}
\lips
   |
 ^^^^"#
    ));
}

#[test]
fn component_command_bibtex() {
    assert_json_snapshot!(complete(
        r#"
%! main.bib
@article{b,
    c = {\LaT }
           |
          ^^^
}"#
    ));
}

#[test]
fn component_environment_simple() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\begin{doc
          |
       ^^^"#
    ));
}

#[test]
fn component_environment_simple_end() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\begin{document}
\end{
     |"#
    ));
}

#[test]
fn component_environment_class() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\documentclass{article}
\begin{thein}
          |
       ^^^^^"#
    ));
}

#[test]
fn component_environment_command_definition() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\newcommand{\foo}{\begin{doc}
                           |
                         ^^^"#
    ));
}

#[test]
fn entry_type_at_empty() {
    assert_json_snapshot!(complete(
        r#"
%! main.bib
@
 |"#
    ));
}

#[test]
fn entry_type_before_preamble() {
    assert_json_snapshot!(complete(
        r#"
%! main.bib
@preamble
 |
 ^^^^^^^^"#
    ));
}

#[test]
fn entry_type_before_string() {
    assert_json_snapshot!(complete(
        r#"
%! main.bib
@string
 |
 ^^^^^^"#
    ));
}

#[test]
fn entry_type_before_article() {
    assert_json_snapshot!(complete(
        r#"
%! main.bib
@article
 |
 ^^^^^^^"#
    ));
}

#[test]
fn entry_type_after_preamble() {
    assert_json_snapshot!(complete(
        r#"
%! main.bib
@preamble{
         |
 ^^^^^^^^"#
    ));
}

#[test]
fn entry_type_after_string() {
    assert_json_snapshot!(complete(
        r#"
%! main.bib
@string{
       |
 ^^^^^^"#
    ));
}

#[test]
fn entry_type_complete_entry() {
    assert_json_snapshot!(complete(
        r#"
%! main.bib
@article{foo, author = {foo}}
   |
 ^^^^^^^"#
    ));
}

#[test]
fn field_empty_entry_open() {
    assert_json_snapshot!(complete(
        r#"
%! main.bib
@article{foo,
             |"#
    ));
}

#[test]
fn field_empty_entry_closed() {
    assert_json_snapshot!(complete(
        r#"
%! main.bib
@article{foo,}
             |"#
    ));
}

#[test]
fn field_entry_field_name() {
    assert_json_snapshot!(complete(
        r#"
%! main.bib
@article{foo, a
               |
              ^"#
    ));
}

#[test]
fn field_entry_two_fields_name_open() {
    assert_json_snapshot!(complete(
        r#"
%! main.bib
@article{foo, author = bar, edit
                             |
                            ^^^^"#
    ));
}

#[test]
fn field_entry_two_fields_name_closed() {
    assert_json_snapshot!(complete(
        r#"
%! main.bib
@article{foo, author = bar, edit}
                             |
                            ^^^^"#
    ));
}

#[test]
fn import_package_open_brace() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\usepackage{lips
             |
            ^^^^"#
    ));
}

#[test]
fn import_package_closed_brace() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\usepackage{lips}
             |
            ^^^^"#
    ));
}

#[test]
fn import_class_open_brace() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\documentclass{art \foo
                |
               ^^^"#
    ));
}

#[test]
fn import_class_closed_brace() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\documentclass{art}
                |
               ^^^"#
    ));
}

#[test]
fn label() {
    assert_json_snapshot!(complete(
        r#"
%! foo.tex
\documentclass{article}

\usepackage{amsmath}
\usepackage{caption}
\usepackage{amsthm}
\newtheorem{lemma}{Lemma}

\begin{document}

\section{Foo}%
\label{sec:foo}

\begin{equation}%
\label{eq:foo}
    1 + 1 = 2
\end{equation}

\begin{equation}%
\label{eq:bar}
    1 + 1 = 2
\end{equation}

\begin{figure}%
\LaTeX{}
\caption{Baz}%
\label{fig:baz}
\end{figure}

\begin{lemma}%
\label{thm:foo}
    1 + 1 = 2
\end{lemma}

\include{bar}

\end{document}

%! bar.tex
\section{Bar}%
\label{sec:bar}

Lorem ipsum dolor sit amet.
\ref{}
     |

%! foo.aux
\relax
\@writefile{lof}{\contentsline {figure}{\numberline {1}{\ignorespaces Baz\relax }}{1}\protected@file@percent }
\providecommand*\caption@xref[2]{\@setref\relax\@undefined{#1}}
\newlabel{fig:baz}{{1}{1}}
\@writefile{toc}{\contentsline {section}{\numberline {1}Foo}{1}\protected@file@percent }
\newlabel{sec:foo}{{1}{1}}
\newlabel{eq:foo}{{1}{1}}
\newlabel{eq:bar}{{2}{1}}
\newlabel{thm:foo}{{1}{1}}
\@input{bar.aux}"#
    ));
}

#[test]
fn theorem_begin() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\newtheorem{lemma}{Lemma}
\begin{lem
        |
       ^^^"#
    ));
}

#[test]
fn theorem_end() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\newtheorem{lemma}{Lemma}
\begin{}
\end{lem
      |
     ^^^"#
    ));
}

#[test]
fn tikz_library_open_brace() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\usepgflibrary{
               |"#
    ));
}

#[test]
fn tikz_library_closed_brace() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\usepgflibrary{}
               |"#
    ));
}

#[test]
fn test_user_command() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\foobar
\fooba
   |
 ^^^^^
\begin{foo}
\end{foo}
\begin{fo}"#
    ));
}

#[test]
fn test_user_environment() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\foobar
\fooba
\begin{foo}
\end{foo}
\begin{fo}
        |
       ^^"#
    ));
}

#[test]
fn test_project_resolution_import() {
    assert_json_snapshot!(complete(
        r#"
%! main.tex
\documentclass{article}
\import{sub}{sub/sub.tex}
\lipsu
     |
 ^^^^^

%! sub/sub.tex
\input{child.tex}

%! sub/child.tex
\usepackage{lipsum}"#
    ));
}

#[test]
fn test_project_resolution_texlabroot() {
    assert_json_snapshot!(complete(
        r#"
%! src/main.tex
\documentclass{article}
\include{src/foo}
\lipsu
     |
 ^^^^^

%! src/foo.tex
\include{src/bar}

%! src/bar.tex
\usepackage{lipsum}

%! .texlabroot"#
    ));
}

#[test]
fn issue_857_1() {
    assert_json_snapshot!(complete(
        r#"
%! bug.tex
\documentclass{article}
\newcommand{\ö}{foo}
\newcommand{\öö}{bar}
\newcommand{\ööabc}{baz}
\begin{document}
\ö
  |
 ^
\end{document}
"#
    ));
}

#[test]
fn issue_864() {
    assert_json_snapshot!(complete(
        r#"
%! bug.tex
\documentclass{article}
\def\あいうえお{}
\begin{document}
\あ
  |
 ^
\end{document}"#
    ))
}

#[test]
fn issue_883() {
    assert_json_snapshot!(complete(
        r#"
%! bug.tex
\begin{doc
          |
       ^^^
% Comment"#
    ))
}
