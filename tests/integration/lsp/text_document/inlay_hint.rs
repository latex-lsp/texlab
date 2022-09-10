use anyhow::Result;
use insta::assert_json_snapshot;
use lsp_types::{
    request::InlayHintRequest, ClientCapabilities, InlayHint, InlayHintParams, Position, Range,
    TextDocumentIdentifier,
};

use crate::lsp::{client::Client, fixture};

fn check(fixture: &str) -> Result<Vec<InlayHint>> {
    let mut client = Client::spawn()?;
    client.initialize(ClientCapabilities::default(), None)?;

    let fixture = fixture::parse(fixture);
    let uri = client.uri(fixture.files[0].name)?;

    for file in fixture.files {
        client.open(file.name, file.lang, file.text)?;
    }

    let actual_hints = client
        .request::<InlayHintRequest>(InlayHintParams {
            text_document: TextDocumentIdentifier::new(uri),
            range: Range::new(Position::new(0, 0), Position::new(12, 0)),
            work_done_progress_params: Default::default(),
        })?
        .unwrap_or_default();

    client.shutdown()?;

    Ok(actual_hints)
}

#[test]
fn label_definition() -> Result<()> {
    assert_json_snapshot!(check(
        r#"
%TEX main.tex
%SRC \documentclass{article}
%SRC \usepackage{caption}
%SRC \begin{document}
%SRC \section{Foo}\label{sec:foo}
%SRC \section{Bar}\label{sec:bar}
%SRC \subsection{Baz}\label{sec:baz}
%SRC \begin{figure}
%SRC     Test
%SRC     \label{fig:qux}
%SRC     \caption{Qux}
%SRC \end{figure}
%SRC \end{document}

%TEX main.aux
%SRC \relax 
%SRC \providecommand*\caption@xref[2]{\@setref\relax\@undefined{#1}}
%SRC \newlabel{fig:qux}{{\caption@xref {fig:qux}{ on input line 15}}{1}}
%SRC \@writefile{lof}{\contentsline {figure}{\numberline {1}{\ignorespaces Qux\relax }}{1}{}\protected@file@percent }
%SRC \@writefile{toc}{\contentsline {section}{\numberline {1}Foo}{1}{}\protected@file@percent }
%SRC \newlabel{sec:foo}{{1}{1}}
%SRC \@writefile{toc}{\contentsline {section}{\numberline {2}Bar}{1}{}\protected@file@percent }
%SRC \newlabel{sec:bar}{{2}{1}}
%SRC \@writefile{toc}{\contentsline {subsection}{\numberline {2.1}Baz}{1}{}\protected@file@percent }
%SRC \newlabel{sec:baz}{{2.1}{1}}
%SRC \gdef \@abspage@last{1}
"#,
    )?);

    Ok(())
}
