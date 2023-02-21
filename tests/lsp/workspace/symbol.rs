use insta::assert_json_snapshot;
use lsp_types::{
    request::WorkspaceSymbolRequest, ClientCapabilities, SymbolInformation, WorkspaceSymbolParams,
    WorkspaceSymbolResponse,
};

use crate::fixture::TestBed;

fn find_symbols(fixture: &str, query: &str) -> Vec<SymbolInformation> {
    let test_bed = TestBed::new(fixture).unwrap();

    test_bed.initialize(ClientCapabilities::default()).unwrap();

    let mut symbols = match test_bed
        .client()
        .send_request::<WorkspaceSymbolRequest>(WorkspaceSymbolParams {
            query: query.to_string(),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        })
        .unwrap()
    {
        Some(WorkspaceSymbolResponse::Flat(symbols)) => symbols,
        Some(WorkspaceSymbolResponse::Nested(_)) => unreachable!(),
        None => Vec::new(),
    };

    for symbol in &mut symbols {
        symbol.location.uri = test_bed.redact(&symbol.location.uri);
    }

    symbols
}

const FIXTURE: &str = r#"
%! main.tex
\documentclass{article}
\usepackage{caption}
\usepackage{amsmath}
\usepackage{amsthm}

\begin{document}

\section{Foo}\label{sec:foo}

\begin{equation}\label{eq:foo}
    Foo
\end{equation}

\section{Bar}\label{sec:bar}

\begin{figure}
    Bar
    \caption{Bar}
    \label{fig:bar}
\end{figure}

\section{Baz}\label{sec:baz}

\begin{enumerate}
    \item\label{itm:foo} Foo
    \item\label{itm:bar} Bar
    \item\label{itm:baz} Baz
\end{enumerate}

\section{Qux}\label{sec:qux}

\newtheorem{lemma}{Lemma}

\begin{lemma}[Qux]\label{thm:qux}
    Qux
\end{lemma}

\end{document}
|

%! main.aux
\relax
\@writefile{lof}{\contentsline {figure}{\numberline {1}{\ignorespaces Bar\relax }}{1}\protected@file@percent }
\providecommand*\caption@xref[2]{\@setref\relax\@undefined{#1}}
\newlabel{fig:bar}{{1}{1}}
\@writefile{toc}{\contentsline {section}{\numberline {1}Foo}{1}\protected@file@percent }
\newlabel{sec:foo}{{1}{1}}
\newlabel{eq:foo}{{1}{1}}
\@writefile{toc}{\contentsline {section}{\numberline {2}Bar}{1}\protected@file@percent }
\newlabel{sec:bar}{{2}{1}}
\@writefile{toc}{\contentsline {section}{\numberline {3}Baz}{1}\protected@file@percent }
\newlabel{sec:baz}{{3}{1}}
\newlabel{itm:foo}{{1}{1}}
\newlabel{itm:bar}{{2}{1}}
\newlabel{itm:baz}{{3}{1}}
\@writefile{toc}{\contentsline {section}{\numberline {4}Qux}{1}\protected@file@percent }
\newlabel{sec:qux}{{4}{1}}
\newlabel{thm:qux}{{1}{1}}

%! main.bib
@article{foo,}

@string{bar = "bar"}"#;

#[test]
fn filter_type_section() {
    assert_json_snapshot!(find_symbols(FIXTURE, "section"));
}

#[test]
fn filter_type_figure() {
    assert_json_snapshot!(find_symbols(FIXTURE, "figure"));
}

#[test]
fn filter_type_item() {
    assert_json_snapshot!(find_symbols(FIXTURE, "item"));
}

#[test]
fn filter_type_math() {
    assert_json_snapshot!(find_symbols(FIXTURE, "math"));
}

#[test]
fn filter_bibtex() {
    assert_json_snapshot!(find_symbols(FIXTURE, "bibtex"));
}
