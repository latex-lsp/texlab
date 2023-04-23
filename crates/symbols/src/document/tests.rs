use base_db::{Config, SymbolConfig};
use insta::assert_debug_snapshot;
use regex::Regex;
use test_utils::fixture::Fixture;

use crate::document_symbols;

#[test]
fn test_enumerate() {
    let fixture = Fixture::parse(
        r#"
%! main.tex
\documentclass{article}

\begin{document}

\begin{enumerate}
    \item\label{it:foo} Foo
    \item\label{it:bar} Bar
    \item[Baz] Baz
    \item[Qux]\label{it:qux} Qux
\end{enumerate}

\end{document}

%! main.aux
\relax
\newlabel{it:foo}{{1}{1}}
\newlabel{it:qux}{{2}{1}}"#,
    );

    let document = fixture.workspace.lookup(&fixture.documents[0].uri).unwrap();
    assert_debug_snapshot!(document_symbols(&fixture.workspace, document));
}

#[test]
fn test_equation() {
    let fixture = Fixture::parse(
        r#"
%! main.tex
\documentclass{article}

\begin{document}

\begin{equation}\label{eq:foo}
    Foo
\end{equation}

\begin{equation}\label{eq:bar}
    Bar
\end{equation}

\begin{equation}
    Baz
\end{equation}

\end{document}

%! main.aux
\relax
\newlabel{eq:foo}{{1}{1}}"#,
    );

    let document = fixture.workspace.lookup(&fixture.documents[0].uri).unwrap();
    assert_debug_snapshot!(document_symbols(&fixture.workspace, document));
}

#[test]
fn test_float() {
    let fixture = Fixture::parse(
        r#"
%! main.tex
\documentclass{article}

\begin{document}

\begin{figure}
    Foo
    \caption{Foo}\label{fig:foo}
\end{figure}

\begin{figure}
    Bar
    \caption{Bar}\label{fig:bar}
\end{figure}

\begin{figure}
    Baz
    \caption{Baz}
\end{figure}

\begin{figure}
    Qux
\end{figure}

\end{document}
|

%! main.aux
\relax
\@writefile{lof}{\contentsline {figure}{\numberline {1}{\ignorespaces Foo}}{1}\protected@file@percent }
\newlabel{fig:foo}{{1}{1}}
\@writefile{lof}{\contentsline {figure}{\numberline {2}{\ignorespaces Bar}}{1}\protected@file@percent }
\@writefile{lof}{\contentsline {figure}{\numberline {3}{\ignorespaces Baz}}{1}\protected@file@percent }"#,
    );

    let document = fixture.workspace.lookup(&fixture.documents[0].uri).unwrap();
    assert_debug_snapshot!(document_symbols(&fixture.workspace, document));
}

#[test]
fn test_section() {
    let fixture = Fixture::parse(
        r#"
%! main.tex
\documentclass{article}

\begin{document}

\section{Foo}

\section{Bar}\label{sec:bar}

\subsection{Baz}\label{sec:baz}

\end{document}
|

%! main.aux
\relax
\@writefile{toc}{\contentsline {section}{\numberline {1}Foo}{1}\protected@file@percent }
\@writefile{toc}{\contentsline {section}{\numberline {2}Bar}{1}\protected@file@percent }
\newlabel{sec:bar}{{2}{1}}"#,
    );

    let document = fixture.workspace.lookup(&fixture.documents[0].uri).unwrap();
    assert_debug_snapshot!(document_symbols(&fixture.workspace, document));
}

#[test]
fn test_theorem() {
    let fixture = Fixture::parse(
        r#"
%! main.tex
\documentclass{article}
\usepackage{amsthm}
\newtheorem{lemma}{Lemma}

\begin{document}

\begin{lemma}[Foo]\label{thm:foo}
    Foo
\end{lemma}

\begin{lemma}\label{thm:bar}
    Bar
\end{lemma}

\begin{lemma}\label{thm:baz}
    Baz
\end{lemma}

\begin{lemma}[Qux]
    Qux
\end{lemma}

\end{document}
|

%! main.aux
\relax
\newlabel{thm:foo}{{1}{1}}
\newlabel{thm:bar}{{2}{1}}"#,
    );

    let document = fixture.workspace.lookup(&fixture.documents[0].uri).unwrap();
    assert_debug_snapshot!(document_symbols(&fixture.workspace, document));
}

#[test]
fn test_allowed_patterns() {
    let mut fixture = Fixture::parse(
        r#"
%! main.tex
\documentclass{article}

\begin{document}

\begin{equation}\label{eq:foo}
    Foo
\end{equation}

\begin{enumerate}
    \item Foo
    \item Bar
\end{enumerate}

\end{document}"#,
    );

    fixture.workspace.set_config(Config {
        symbols: SymbolConfig {
            allowed_patterns: vec![
                Regex::new("Item").unwrap(),
                Regex::new("Enumerate").unwrap(),
            ],
            ignored_patterns: Vec::new(),
        },
        ..Config::default()
    });

    let document = fixture.workspace.lookup(&fixture.documents[0].uri).unwrap();
    assert_debug_snapshot!(document_symbols(&fixture.workspace, document));
}

#[test]
fn test_ignored_patterns() {
    let mut fixture = Fixture::parse(
        r#"
%! main.tex
\documentclass{article}

\begin{document}

\begin{equation}\label{eq:foo}
    Foo
\end{equation}

\begin{enumerate}
    \item Foo
    \item Bar
\end{enumerate}

\end{document}"#,
    );

    fixture.workspace.set_config(Config {
        symbols: SymbolConfig {
            ignored_patterns: vec![
                Regex::new("Item").unwrap(),
                Regex::new("Enumerate").unwrap(),
            ],
            allowed_patterns: Vec::new(),
        },
        ..Config::default()
    });

    let document = fixture.workspace.lookup(&fixture.documents[0].uri).unwrap();
    assert_debug_snapshot!(document_symbols(&fixture.workspace, document));
}
