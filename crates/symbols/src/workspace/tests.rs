use insta::assert_debug_snapshot;
use test_utils::fixture::Fixture;

use crate::workspace_symbols;

static FIXTURE: &str = r#"
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
    let fixture = Fixture::parse(FIXTURE);
    assert_debug_snapshot!(workspace_symbols(&fixture.workspace, "section"));
}

#[test]
fn filter_type_figure() {
    let fixture = Fixture::parse(FIXTURE);
    assert_debug_snapshot!(workspace_symbols(&fixture.workspace, "figure"));
}

#[test]
fn filter_type_item() {
    let fixture = Fixture::parse(FIXTURE);
    assert_debug_snapshot!(workspace_symbols(&fixture.workspace, "item"));
}

#[test]
fn filter_type_math() {
    let fixture = Fixture::parse(FIXTURE);
    assert_debug_snapshot!(workspace_symbols(&fixture.workspace, "math"));
}

#[test]
fn filter_bibtex() {
    let fixture = Fixture::parse(FIXTURE);
    assert_debug_snapshot!(workspace_symbols(&fixture.workspace, "bibtex"));
}
