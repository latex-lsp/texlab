use insta::{assert_json_snapshot, internals::Redaction};
use lsp_types::{
    notification::DidChangeConfiguration, request::DocumentSymbolRequest,
    DidChangeConfigurationParams, DocumentSymbolParams, DocumentSymbolResponse,
    TextDocumentIdentifier, Url,
};

use crate::tests::{client::Client, fixture};

struct SymbolResult {
    response: Option<DocumentSymbolResponse>,
    uri_redaction: Redaction,
}

fn find_symbols(
    fixture: &str,
    client_capabilities: serde_json::Value,
    settings: serde_json::Value,
) -> SymbolResult {
    let mut client = Client::spawn();
    client.initialize(serde_json::from_value(client_capabilities).unwrap(), None);

    let fixture = fixture::parse(fixture);
    let file = fixture.files.into_iter().next().unwrap();
    client.open(file.name, file.lang, file.text);

    client.notify::<DidChangeConfiguration>(DidChangeConfigurationParams { settings });

    let response = client
        .request::<DocumentSymbolRequest>(DocumentSymbolParams {
            text_document: TextDocumentIdentifier::new(client.uri(file.name)),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        })
        .unwrap();

    let result = client.shutdown();

    let uri = Url::from_directory_path(result.directory.path()).unwrap();
    let uri_redaction = insta::dynamic_redaction(move |content, _path| {
        content.as_str().unwrap().replace(uri.as_str(), "[tmp]/")
    });

    SymbolResult {
        response,
        uri_redaction,
    }
}

macro_rules! assert_symbols {
    ($result:expr) => {
        let result = $result;
        assert_json_snapshot!(result.response, {
            "[].location.uri" => result.uri_redaction
        });
    };
}

#[test]
fn enumerate_nested() {
    assert_symbols!(find_symbols(
        r#"
%TEX main.tex
%SRC \documentclass{article}
%SRC 
%SRC \begin{document}
%SRC 
%SRC \begin{enumerate}
%SRC     \item\label{it:foo} Foo
%SRC     \item\label{it:bar} Bar
%SRC     \item[Baz] Baz
%SRC     \item[Qux]\label{it:qux} Qux
%SRC \end{enumerate}
%SRC 
%SRC \end{document}

%TEX main.aux
%SRC \relax
%SRC \newlabel{it:foo}{{1}{1}}
%SRC \newlabel{it:qux}{{2}{1}}
"#,
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
    assert_symbols!(find_symbols(
        r#"
%TEX main.tex
%SRC \documentclass{article}
%SRC 
%SRC \begin{document}
%SRC 
%SRC \begin{enumerate}
%SRC     \item\label{it:foo} Foo
%SRC     \item\label{it:bar} Bar
%SRC     \item[Baz] Baz
%SRC     \item[Qux]\label{it:qux} Qux
%SRC \end{enumerate}
%SRC 
%SRC \end{document}

%TEX main.aux
%SRC \relax
%SRC \newlabel{it:foo}{{1}{1}}
%SRC \newlabel{it:qux}{{2}{1}}
"#,
        serde_json::json!({}),
        serde_json::Value::Null,
    ));
}

#[test]
fn equation_nested() {
    assert_symbols!(find_symbols(
        r#"
%TEX main.tex
%SRC \documentclass{article}
%SRC 
%SRC \begin{document}
%SRC 
%SRC \begin{equation}\label{eq:foo}
%SRC     Foo
%SRC \end{equation}
%SRC 
%SRC \begin{equation}\label{eq:bar}
%SRC     Bar
%SRC \end{equation}
%SRC 
%SRC \begin{equation}
%SRC     Baz
%SRC \end{equation}
%SRC 
%SRC \end{document}

%TEX main.aux
%SRC \relax
%SRC \newlabel{eq:foo}{{1}{1}}
"#,
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
    assert_symbols!(find_symbols(
        r#"
%TEX main.tex
%SRC \documentclass{article}
%SRC 
%SRC \begin{document}
%SRC 
%SRC \begin{equation}\label{eq:foo}
%SRC     Foo
%SRC \end{equation}
%SRC 
%SRC \begin{equation}\label{eq:bar}
%SRC     Bar
%SRC \end{equation}
%SRC 
%SRC \begin{equation}
%SRC     Baz
%SRC \end{equation}
%SRC 
%SRC \end{document}

%TEX main.aux
%SRC \relax
%SRC \newlabel{eq:foo}{{1}{1}}
"#,
        serde_json::json!({}),
        serde_json::Value::Null,
    ));
}

#[test]
fn float_nested() {
    assert_symbols!(find_symbols(
        r#"
%TEX main.tex
%SRC \documentclass{article}
%SRC 
%SRC \begin{document}
%SRC 
%SRC \begin{figure}
%SRC     Foo
%SRC     \caption{Foo}\label{fig:foo}
%SRC \end{figure}
%SRC 
%SRC \begin{figure}
%SRC     Bar
%SRC     \caption{Bar}\label{fig:bar}
%SRC \end{figure}
%SRC 
%SRC \begin{figure}
%SRC     Baz
%SRC     \caption{Baz}
%SRC \end{figure}
%SRC 
%SRC \begin{figure}
%SRC     Qux
%SRC \end{figure}
%SRC 
%SRC \end{document}

%TEX main.aux
%SRC \relax
%SRC \@writefile{lof}{\contentsline {figure}{\numberline {1}{\ignorespaces Foo}}{1}\protected@file@percent }
%SRC \newlabel{fig:foo}{{1}{1}}
%SRC \@writefile{lof}{\contentsline {figure}{\numberline {2}{\ignorespaces Bar}}{1}\protected@file@percent }
%SRC \@writefile{lof}{\contentsline {figure}{\numberline {3}{\ignorespaces Baz}}{1}\protected@file@percent }
"#,
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
    assert_symbols!(find_symbols(
        r#"
%TEX main.tex
%SRC \documentclass{article}
%SRC 
%SRC \begin{document}
%SRC 
%SRC \begin{figure}
%SRC     Foo
%SRC     \caption{Foo}\label{fig:foo}
%SRC \end{figure}
%SRC 
%SRC \begin{figure}
%SRC     Bar
%SRC     \caption{Bar}\label{fig:bar}
%SRC \end{figure}
%SRC 
%SRC \begin{figure}
%SRC     Baz
%SRC     \caption{Baz}
%SRC \end{figure}
%SRC 
%SRC \begin{figure}
%SRC     Qux
%SRC \end{figure}
%SRC 
%SRC \end{document}

%TEX main.aux
%SRC \relax
%SRC \@writefile{lof}{\contentsline {figure}{\numberline {1}{\ignorespaces Foo}}{1}\protected@file@percent }
%SRC \newlabel{fig:foo}{{1}{1}}
%SRC \@writefile{lof}{\contentsline {figure}{\numberline {2}{\ignorespaces Bar}}{1}\protected@file@percent }
%SRC \@writefile{lof}{\contentsline {figure}{\numberline {3}{\ignorespaces Baz}}{1}\protected@file@percent }
"#,
        serde_json::json!({}),
        serde_json::Value::Null,
    ));
}

#[test]
fn section_nested() {
    assert_symbols!(find_symbols(
        r#"
%TEX main.tex
%SRC \documentclass{article}
%SRC 
%SRC \begin{document}
%SRC 
%SRC \section{Foo}
%SRC 
%SRC \section{Bar}\label{sec:bar}
%SRC 
%SRC \subsection{Baz}\label{sec:baz}
%SRC 
%SRC \end{document}

%TEX main.aux
%SRC \relax
%SRC \@writefile{toc}{\contentsline {section}{\numberline {1}Foo}{1}\protected@file@percent }
%SRC \@writefile{toc}{\contentsline {section}{\numberline {2}Bar}{1}\protected@file@percent }
%SRC \newlabel{sec:bar}{{2}{1}}
"#,
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
    assert_symbols!(find_symbols(
        r#"
%TEX main.tex
%SRC \documentclass{article}
%SRC 
%SRC \begin{document}
%SRC 
%SRC \section{Foo}
%SRC 
%SRC \section{Bar}\label{sec:bar}
%SRC 
%SRC \subsection{Baz}\label{sec:baz}
%SRC 
%SRC \end{document}

%TEX main.aux
%SRC \relax
%SRC \@writefile{toc}{\contentsline {section}{\numberline {1}Foo}{1}\protected@file@percent }
%SRC \@writefile{toc}{\contentsline {section}{\numberline {2}Bar}{1}\protected@file@percent }
%SRC \newlabel{sec:bar}{{2}{1}}
"#,
        serde_json::json!({}),
        serde_json::Value::Null,
    ));
}

#[test]
fn theorem_nested() {
    assert_symbols!(find_symbols(
        r#"
%TEX main.tex
%SRC \documentclass{article}
%SRC \usepackage{amsthm}
%SRC \newtheorem{lemma}{Lemma}
%SRC 
%SRC \begin{document}
%SRC 
%SRC \begin{lemma}[Foo]\label{thm:foo}
%SRC     Foo
%SRC \end{lemma}
%SRC 
%SRC \begin{lemma}\label{thm:bar}
%SRC     Bar
%SRC \end{lemma}
%SRC 
%SRC \begin{lemma}\label{thm:baz}
%SRC     Baz
%SRC \end{lemma}
%SRC 
%SRC \begin{lemma}[Qux]
%SRC     Qux
%SRC \end{lemma}
%SRC 
%SRC \end{document}

%TEX main.aux
%SRC \relax
%SRC \newlabel{thm:foo}{{1}{1}}
%SRC \newlabel{thm:bar}{{2}{1}}
"#,
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
    assert_symbols!(find_symbols(
        r#"
%TEX main.tex
%SRC \documentclass{article}
%SRC \usepackage{amsthm}
%SRC \newtheorem{lemma}{Lemma}
%SRC 
%SRC \begin{document}
%SRC 
%SRC \begin{lemma}[Foo]\label{thm:foo}
%SRC     Foo
%SRC \end{lemma}
%SRC 
%SRC \begin{lemma}\label{thm:bar}
%SRC     Bar
%SRC \end{lemma}
%SRC 
%SRC \begin{lemma}\label{thm:baz}
%SRC     Baz
%SRC \end{lemma}
%SRC 
%SRC \begin{lemma}[Qux]
%SRC     Qux
%SRC \end{lemma}
%SRC 
%SRC \end{document}

%TEX main.aux
%SRC \relax
%SRC \newlabel{thm:foo}{{1}{1}}
%SRC \newlabel{thm:bar}{{2}{1}}
"#,
        serde_json::json!({}),
        serde_json::Value::Null,
    ));
}

#[test]
fn ignored_patterns() {
    assert_symbols!(find_symbols(
        r#"
%TEX main.tex
%SRC \documentclass{article}
%SRC 
%SRC \begin{document}
%SRC 
%SRC \begin{equation}\label{eq:foo}
%SRC     Foo
%SRC \end{equation}
%SRC  
%SRC \begin{enumerate}
%SRC     \item Foo
%SRC     \item Bar
%SRC \end{enumerate}
%SRC 
%SRC \end{document}
"#,
        serde_json::json!({}),
        serde_json::json!({
            "symbols": {
                "ignoredPatterns": ["Item", "Enumerate"]
            }
        }),
    ));
}
