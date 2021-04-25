use anyhow::Result;
use insta::assert_json_snapshot;
use lsp_types::{
    ClientCapabilities, DocumentSymbolClientCapabilities, TextDocumentClientCapabilities, Url,
};

use crate::common::ServerTester;

fn nested_symbol_capabilities() -> ClientCapabilities {
    ClientCapabilities {
        text_document: Some(TextDocumentClientCapabilities {
            document_symbol: Some(DocumentSymbolClientCapabilities {
                hierarchical_document_symbol_support: Some(true),
                ..DocumentSymbolClientCapabilities::default()
            }),
            ..TextDocumentClientCapabilities::default()
        }),
        ..ClientCapabilities::default()
    }
}

#[test]
fn test_enumerate_nested() -> Result<()> {
    let server = ServerTester::launch_new_instance()?;
    server.initialize(nested_symbol_capabilities(), None)?;
    let uri = Url::parse("http://www.example.com/main.tex")?;
    server.open_memory(
        uri.clone(),
        r#"
            \documentclass{article}

            \begin{document}

            \begin{enumerate}
                \item\label{it:foo} Foo
                \item\label{it:bar} Bar
                \item[Baz] Baz
                \item[Qux]\label{it:qux} Qux
            \end{enumerate}

            \end{document}
        "#,
        "latex",
    )?;
    server.open_memory(
        Url::parse("http://www.example.com/main.aux")?,
        r#"
            \relax
            \newlabel{it:foo}{{1}{1}}
            \newlabel{it:qux}{{2}{1}}
        "#,
        "latex",
    )?;
    assert_json_snapshot!(server.find_document_symbols(uri)?);
    Ok(())
}

#[test]
fn test_enumerate_flat() -> Result<()> {
    let server = ServerTester::launch_new_instance()?;
    server.initialize(ClientCapabilities::default(), None)?;
    let uri = Url::parse("http://www.example.com/main.tex")?;
    server.open_memory(
        uri.clone(),
        r#"
            \documentclass{article}

            \begin{document}

            \begin{enumerate}
                \item\label{it:foo} Foo
                \item\label{it:bar} Bar
                \item[Baz] Baz
                \item[Qux]\label{it:qux} Qux
            \end{enumerate}

            \end{document}
        "#,
        "latex",
    )?;
    server.open_memory(
        Url::parse("http://www.example.com/main.aux")?,
        r#"
            \relax
            \newlabel{it:foo}{{1}{1}}
            \newlabel{it:qux}{{2}{1}}
        "#,
        "latex",
    )?;
    assert_json_snapshot!(server.find_document_symbols(uri)?);
    Ok(())
}

#[test]
fn test_equation_nested() -> Result<()> {
    let server = ServerTester::launch_new_instance()?;
    server.initialize(nested_symbol_capabilities(), None)?;
    let uri = Url::parse("http://www.example.com/main.tex")?;
    server.open_memory(
        uri.clone(),
        r#"
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
        "#,
        "latex",
    )?;
    server.open_memory(
        Url::parse("http://www.example.com/main.aux")?,
        r#"
            \relax
            \newlabel{eq:foo}{{1}{1}}
        "#,
        "latex",
    )?;
    assert_json_snapshot!(server.find_document_symbols(uri)?);
    Ok(())
}

#[test]
fn test_equation_flat() -> Result<()> {
    let server = ServerTester::launch_new_instance()?;
    server.initialize(ClientCapabilities::default(), None)?;
    let uri = Url::parse("http://www.example.com/main.tex")?;
    server.open_memory(
        uri.clone(),
        r#"
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
        "#,
        "latex",
    )?;
    server.open_memory(
        Url::parse("http://www.example.com/main.aux")?,
        r#"
            \relax
            \newlabel{eq:foo}{{1}{1}}
        "#,
        "latex",
    )?;
    assert_json_snapshot!(server.find_document_symbols(uri)?);
    Ok(())
}

#[test]
fn test_float_nested() -> Result<()> {
    let server = ServerTester::launch_new_instance()?;
    server.initialize(nested_symbol_capabilities(), None)?;
    let uri = Url::parse("http://www.example.com/main.tex")?;
    server.open_memory(
        uri.clone(),
        r#"
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
        "#,
        "latex",
    )?;
    server.open_memory(
        Url::parse("http://www.example.com/main.aux")?,
        r#"
            \relax
            \@writefile{lof}{\contentsline {figure}{\numberline {1}{\ignorespaces Foo}}{1}\protected@file@percent }
            \newlabel{fig:foo}{{1}{1}}
            \@writefile{lof}{\contentsline {figure}{\numberline {2}{\ignorespaces Bar}}{1}\protected@file@percent }
            \@writefile{lof}{\contentsline {figure}{\numberline {3}{\ignorespaces Baz}}{1}\protected@file@percent }
        "#,
        "latex",
    )?;
    assert_json_snapshot!(server.find_document_symbols(uri)?);
    Ok(())
}

#[test]
fn test_float_flat() -> Result<()> {
    let server = ServerTester::launch_new_instance()?;
    server.initialize(ClientCapabilities::default(), None)?;
    let uri = Url::parse("http://www.example.com/main.tex")?;
    server.open_memory(
        uri.clone(),
        r#"
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
        "#,
        "latex",
    )?;
    server.open_memory(
        Url::parse("http://www.example.com/main.aux")?,
        r#"
            \relax
            \@writefile{lof}{\contentsline {figure}{\numberline {1}{\ignorespaces Foo}}{1}\protected@file@percent }
            \newlabel{fig:foo}{{1}{1}}
            \@writefile{lof}{\contentsline {figure}{\numberline {2}{\ignorespaces Bar}}{1}\protected@file@percent }
            \@writefile{lof}{\contentsline {figure}{\numberline {3}{\ignorespaces Baz}}{1}\protected@file@percent }
        "#,
        "latex",
    )?;
    assert_json_snapshot!(server.find_document_symbols(uri)?);
    Ok(())
}

#[test]
fn test_section_nested() -> Result<()> {
    let server = ServerTester::launch_new_instance()?;
    server.initialize(nested_symbol_capabilities(), None)?;
    let uri = Url::parse("http://www.example.com/main.tex")?;
    server.open_memory(
        uri.clone(),
        r#"
            \documentclass{article}

            \begin{document}

            \section{Foo}

            \section{Bar}\label{sec:bar}

            \subsection{Baz}\label{sec:baz}

            \end{document}
        "#,
        "latex",
    )?;
    server.open_memory(
        Url::parse("http://www.example.com/main.aux")?,
        r#"
            \relax
            \@writefile{toc}{\contentsline {section}{\numberline {1}Foo}{1}\protected@file@percent }
            \@writefile{toc}{\contentsline {section}{\numberline {2}Bar}{1}\protected@file@percent }
            \newlabel{sec:bar}{{2}{1}}
        "#,
        "latex",
    )?;
    assert_json_snapshot!(server.find_document_symbols(uri)?);
    Ok(())
}

#[test]
fn test_section_flat() -> Result<()> {
    let server = ServerTester::launch_new_instance()?;
    server.initialize(ClientCapabilities::default(), None)?;
    let uri = Url::parse("http://www.example.com/main.tex")?;
    server.open_memory(
        uri.clone(),
        r#"
            \documentclass{article}

            \begin{document}

            \section{Foo}

            \section{Bar}\label{sec:bar}

            \subsection{Baz}\label{sec:baz}

            \end{document}
        "#,
        "latex",
    )?;
    server.open_memory(
        Url::parse("http://www.example.com/main.aux")?,
        r#"
            \relax
            \@writefile{toc}{\contentsline {section}{\numberline {1}Foo}{1}\protected@file@percent }
            \@writefile{toc}{\contentsline {section}{\numberline {2}Bar}{1}\protected@file@percent }
            \newlabel{sec:bar}{{2}{1}}
        "#,
        "latex",
    )?;
    assert_json_snapshot!(server.find_document_symbols(uri)?);
    Ok(())
}

#[test]
fn test_theorem_nested() -> Result<()> {
    let server = ServerTester::launch_new_instance()?;
    server.initialize(nested_symbol_capabilities(), None)?;
    let uri = Url::parse("http://www.example.com/main.tex")?;
    server.open_memory(
        uri.clone(),
        r#"
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
        "#,
        "latex",
    )?;
    server.open_memory(
        Url::parse("http://www.example.com/main.aux")?,
        r#"
            \relax
            \newlabel{thm:foo}{{1}{1}}
            \newlabel{thm:bar}{{2}{1}}
        "#,
        "latex",
    )?;
    assert_json_snapshot!(server.find_document_symbols(uri)?);
    Ok(())
}

#[test]
fn test_theorem_flat() -> Result<()> {
    let server = ServerTester::launch_new_instance()?;
    server.initialize(ClientCapabilities::default(), None)?;
    let uri = Url::parse("http://www.example.com/main.tex")?;
    server.open_memory(
        uri.clone(),
        r#"
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
        "#,
        "latex",
    )?;
    server.open_memory(
        Url::parse("http://www.example.com/main.aux")?,
        r#"
            \relax
            \newlabel{thm:foo}{{1}{1}}
            \newlabel{thm:bar}{{2}{1}}
        "#,
        "latex",
    )?;
    assert_json_snapshot!(server.find_document_symbols(uri)?);
    Ok(())
}
