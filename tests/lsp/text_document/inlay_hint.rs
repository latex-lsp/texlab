use insta::assert_json_snapshot;
use lsp_types::{
    request::InlayHintRequest, ClientCapabilities, InlayHint, InlayHintParams, Position, Range,
};

use crate::fixture::TestBed;

fn find_hints(fixture: &str) -> Vec<InlayHint> {
    let test_bed = TestBed::new(fixture).unwrap();
    test_bed.initialize(ClientCapabilities::default()).unwrap();

    let cursor = test_bed.cursor().unwrap();

    test_bed
        .client()
        .send_request::<InlayHintRequest>(InlayHintParams {
            text_document: cursor.text_document,
            range: Range::new(Position::new(0, 0), cursor.position),
            work_done_progress_params: Default::default(),
        })
        .unwrap()
        .unwrap_or_default()
}

#[test]
fn label_definition() {
    assert_json_snapshot!(find_hints(
        r#"
%! main.tex
\documentclass{article}
\usepackage{caption}
\begin{document}
\section{Foo}\label{sec:foo}
\section{Bar}\label{sec:bar}
\subsection{Baz}\label{sec:baz}
\begin{figure}
    Test
    \label{fig:qux}
    \caption{Qux}
\end{figure}
\end{document}
              |

%! main.aux
\relax 
\providecommand*\caption@xref[2]{\@setref\relax\@undefined{#1}}
\newlabel{fig:qux}{{\caption@xref {fig:qux}{ on input line 15}}{1}}
\@writefile{lof}{\contentsline {figure}{\numberline {1}{\ignorespaces Qux\relax }}{1}{}\protected@file@percent }
\@writefile{toc}{\contentsline {section}{\numberline {1}Foo}{1}{}\protected@file@percent }
\newlabel{sec:foo}{{1}{1}}
\@writefile{toc}{\contentsline {section}{\numberline {2}Bar}{1}{}\protected@file@percent }
\newlabel{sec:bar}{{2}{1}}
\@writefile{toc}{\contentsline {subsection}{\numberline {2.1}Baz}{1}{}\protected@file@percent }
\newlabel{sec:baz}{{2.1}{1}}
\gdef \@abspage@last{1}"#,
    ));
}
