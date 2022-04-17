use anyhow::Result;
use insta::assert_json_snapshot;
use lsp_types::{ClientCapabilities, SymbolInformation, Url};

use crate::common::ServerTester;

fn run(query: &str) -> Result<Vec<SymbolInformation>> {
    let server = ServerTester::launch_new_instance()?;
    server.initialize(ClientCapabilities::default(), None)?;
    let uri = Url::parse("http://www.example.com/main.tex")?;
    server.open_memory(
        uri,
        r#"
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
        "#,
        "latex",
    )?;
    server.open_memory(
        Url::parse("http://www.example.com/main.aux")?,
        r#"
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
        "#,
        "latex",
    )?;
    server.open_memory(
        Url::parse("http://www.example.com/main.bib")?,
        r#"
            @article{foo,}

            @string{bar = "bar"}
        "#,
        "bibtex",
    )?;

    server.find_workspace_symbols(query)
}

#[test]
fn test_filter_type_section() -> Result<()> {
    assert_json_snapshot!(run("section")?);
    Ok(())
}

#[test]
fn test_filter_type_figure() -> Result<()> {
    assert_json_snapshot!(run("figure")?);
    Ok(())
}

#[test]
fn test_filter_type_item() -> Result<()> {
    assert_json_snapshot!(run("item")?);
    Ok(())
}

#[test]
fn test_filter_type_math() -> Result<()> {
    assert_json_snapshot!(run("math")?);
    Ok(())
}

#[test]
fn test_filter_bibtex() -> Result<()> {
    assert_json_snapshot!(run("bibtex")?);
    Ok(())
}
