use insta::assert_json_snapshot;
use lsp_types::{
    notification::DidChangeConfiguration, request::DocumentSymbolRequest,
    DidChangeConfigurationParams, DocumentSymbolParams, DocumentSymbolResponse,
};

use crate::fixture::TestBed;

fn find_symbols(
    fixture: &str,
    capabilities: serde_json::Value,
    settings: serde_json::Value,
) -> DocumentSymbolResponse {
    let test_bed = TestBed::new(fixture).unwrap();

    test_bed
        .initialize(serde_json::from_value(capabilities).unwrap())
        .unwrap();

    test_bed
        .client()
        .send_notification::<DidChangeConfiguration>(DidChangeConfigurationParams { settings })
        .unwrap();

    let text_document = test_bed.cursor().unwrap().text_document;

    let mut response = test_bed
        .client()
        .send_request::<DocumentSymbolRequest>(DocumentSymbolParams {
            text_document,
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        })
        .unwrap()
        .unwrap_or_else(|| DocumentSymbolResponse::Flat(vec![]));

    if let DocumentSymbolResponse::Flat(symbols) = &mut response {
        for symbol in symbols {
            symbol.location.uri = test_bed.redact(&symbol.location.uri);
        }
    }

    response
}

#[test]
fn enumerate_nested() {
    assert_json_snapshot!(find_symbols(
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
|

%! main.aux
\relax
\newlabel{it:foo}{{1}{1}}
\newlabel{it:qux}{{2}{1}}"#,
        serde_json::json!({
            "textDocument": {
                "documentSymbol": {
                    "hierarchicalDocumentSymbolSupport": true,
                },
            },
        }),
        serde_json::Value::Null,
    ));
}

#[test]
fn enumerate_flat() {
    assert_json_snapshot!(find_symbols(
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
|

%! main.aux
\relax
\newlabel{it:foo}{{1}{1}}
\newlabel{it:qux}{{2}{1}}"#,
        serde_json::json!({}),
        serde_json::Value::Null,
    ));
}

#[test]
fn equation_nested() {
    assert_json_snapshot!(find_symbols(
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
|

%! main.aux
\relax
\newlabel{eq:foo}{{1}{1}}"#,
        serde_json::json!({
            "textDocument": {
                "documentSymbol": {
                    "hierarchicalDocumentSymbolSupport": true,
                },
            },
        }),
        serde_json::Value::Null,
    ));
}

#[test]
fn equation_flat() {
    assert_json_snapshot!(find_symbols(
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
|

%! main.aux
\relax
\newlabel{eq:foo}{{1}{1}}"#,
        serde_json::json!({}),
        serde_json::Value::Null,
    ));
}

#[test]
fn float_nested() {
    assert_json_snapshot!(find_symbols(
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
        serde_json::json!({
            "textDocument": {
                "documentSymbol": {
                    "hierarchicalDocumentSymbolSupport": true,
                },
            },
        }),
        serde_json::Value::Null,
    ));
}

#[test]
fn float_flat() {
    assert_json_snapshot!(find_symbols(
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
        serde_json::json!({}),
        serde_json::Value::Null,
    ));
}

#[test]
fn section_nested() {
    assert_json_snapshot!(find_symbols(
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
        serde_json::json!({
            "textDocument": {
                "documentSymbol": {
                    "hierarchicalDocumentSymbolSupport": true,
                },
            },
        }),
        serde_json::Value::Null,
    ));
}

#[test]
fn section_flat() {
    assert_json_snapshot!(find_symbols(
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
        serde_json::json!({}),
        serde_json::Value::Null,
    ));
}

#[test]
fn theorem_nested() {
    assert_json_snapshot!(find_symbols(
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
        serde_json::json!({
            "textDocument": {
                "documentSymbol": {
                    "hierarchicalDocumentSymbolSupport": true,
                },
            },
        }),
        serde_json::Value::Null,
    ));
}

#[test]
fn theorem_flat() {
    assert_json_snapshot!(find_symbols(
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
        serde_json::json!({}),
        serde_json::Value::Null,
    ));
}

#[test]
fn ignored_patterns() {
    assert_json_snapshot!(find_symbols(
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

\end{document}
|"#,
        serde_json::json!({}),
        serde_json::json!({
            "symbols": {
                "ignoredPatterns": ["Item", "Enumerate"]
            }
        }),
    ));
}
