use insta::assert_debug_snapshot;
use lsp_types::{
    request::WorkspaceSymbolRequest, ClientCapabilities, WorkspaceSymbolParams,
    WorkspaceSymbolResponse,
};

use crate::fixture::TestBed;

#[test]
fn test_smoke() {
    let test_bed = TestBed::new(
        r#"
%! main.tex
\documentclass{article}
\usepackage{caption}
\usepackage{amsmath}
\usepackage{amsthm}

\begin{document}

\section{Foo}\label{sec:foo}
Foo

\section{Bar}
Bar

\end{document}
"#,
    )
    .unwrap();

    test_bed.initialize(ClientCapabilities::default()).unwrap();

    let Some(WorkspaceSymbolResponse::Flat(mut symbols)) = test_bed
        .client()
        .send_request::<WorkspaceSymbolRequest>(WorkspaceSymbolParams {
            query: String::new(),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        })
        .unwrap() else { unreachable!() };

    for symbol in &mut symbols {
        symbol.location.uri = test_bed.redact(&symbol.location.uri);
    }

    assert_debug_snapshot!(symbols);
}
