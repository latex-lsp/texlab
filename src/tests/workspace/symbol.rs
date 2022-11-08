use anyhow::Result;
use insta::{assert_json_snapshot, internals::Redaction};
use lsp_types::{
    request::WorkspaceSymbol, ClientCapabilities, SymbolInformation, Url, WorkspaceSymbolParams,
};

use crate::tests::{client::Client, fixture};

struct SymbolResult {
    actual_symbols: Vec<SymbolInformation>,
    uri_redaction: Redaction,
}

fn find_symbols(fixture: &str, query: &str) -> Result<SymbolResult> {
    let mut client = Client::spawn()?;
    client.initialize(ClientCapabilities::default(), None)?;

    let fixture = fixture::parse(fixture);
    for file in fixture.files {
        client.open(file.name, file.lang, file.text)?;
    }

    let actual_symbols = client
        .request::<WorkspaceSymbol>(WorkspaceSymbolParams {
            query: query.to_string(),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        })?
        .unwrap_or_default();

    let result = client.shutdown()?;

    let uri = Url::from_directory_path(result.directory.path()).unwrap();
    let uri_redaction = insta::dynamic_redaction(move |content, _path| {
        content.as_str().unwrap().replace(uri.as_str(), "[tmp]/")
    });

    Ok(SymbolResult {
        actual_symbols,
        uri_redaction,
    })
}

macro_rules! assert_symbols {
    ($result:expr) => {
        let result = $result;
        assert_json_snapshot!(result.actual_symbols, {
            "[].location.uri" => result.uri_redaction,
            "[]" => insta::sorted_redaction()
        });
    };
}

const FIXTURE: &str = r#"
%TEX main.tex
%SRC \documentclass{article}
%SRC \usepackage{caption}
%SRC \usepackage{amsmath}
%SRC \usepackage{amsthm}
%SRC 
%SRC \begin{document}
%SRC 
%SRC \section{Foo}\label{sec:foo}
%SRC 
%SRC \begin{equation}\label{eq:foo}
%SRC     Foo
%SRC \end{equation}
%SRC 
%SRC \section{Bar}\label{sec:bar}
%SRC 
%SRC \begin{figure}
%SRC     Bar
%SRC     \caption{Bar}
%SRC     \label{fig:bar}
%SRC \end{figure}
%SRC 
%SRC \section{Baz}\label{sec:baz}
%SRC 
%SRC \begin{enumerate}
%SRC     \item\label{itm:foo} Foo
%SRC     \item\label{itm:bar} Bar
%SRC     \item\label{itm:baz} Baz
%SRC \end{enumerate}
%SRC 
%SRC \section{Qux}\label{sec:qux}
%SRC 
%SRC \newtheorem{lemma}{Lemma}
%SRC 
%SRC \begin{lemma}[Qux]\label{thm:qux}
%SRC     Qux
%SRC \end{lemma}
%SRC 
%SRC \end{document}

%TEX main.aux
%SRC \relax
%SRC \@writefile{lof}{\contentsline {figure}{\numberline {1}{\ignorespaces Bar\relax }}{1}\protected@file@percent }
%SRC \providecommand*\caption@xref[2]{\@setref\relax\@undefined{#1}}
%SRC \newlabel{fig:bar}{{1}{1}}
%SRC \@writefile{toc}{\contentsline {section}{\numberline {1}Foo}{1}\protected@file@percent }
%SRC \newlabel{sec:foo}{{1}{1}}
%SRC \newlabel{eq:foo}{{1}{1}}
%SRC \@writefile{toc}{\contentsline {section}{\numberline {2}Bar}{1}\protected@file@percent }
%SRC \newlabel{sec:bar}{{2}{1}}
%SRC \@writefile{toc}{\contentsline {section}{\numberline {3}Baz}{1}\protected@file@percent }
%SRC \newlabel{sec:baz}{{3}{1}}
%SRC \newlabel{itm:foo}{{1}{1}}
%SRC \newlabel{itm:bar}{{2}{1}}
%SRC \newlabel{itm:baz}{{3}{1}}
%SRC \@writefile{toc}{\contentsline {section}{\numberline {4}Qux}{1}\protected@file@percent }
%SRC \newlabel{sec:qux}{{4}{1}}
%SRC \newlabel{thm:qux}{{1}{1}}

%BIB main.bib
%SRC @article{foo,}
%SRC 
%SRC @string{bar = "bar"}"#;

#[test]
fn filter_type_section() -> Result<()> {
    assert_symbols!(find_symbols(FIXTURE, "section")?);
    Ok(())
}

#[test]
fn filter_type_figure() -> Result<()> {
    assert_symbols!(find_symbols(FIXTURE, "figure")?);
    Ok(())
}

#[test]
fn filter_type_item() -> Result<()> {
    assert_symbols!(find_symbols(FIXTURE, "item")?);
    Ok(())
}

#[test]
fn filter_type_math() -> Result<()> {
    assert_symbols!(find_symbols(FIXTURE, "math")?);
    Ok(())
}

#[test]
fn filter_bibtex() -> Result<()> {
    assert_symbols!(find_symbols(FIXTURE, "bibtex")?);
    Ok(())
}
