use insta::assert_json_snapshot;
use lsp_types::{
    request::FoldingRangeRequest, ClientCapabilities, FoldingRange, FoldingRangeParams,
};

use crate::fixture::TestBed;

fn find_foldings(fixture: &str) -> Vec<FoldingRange> {
    let test_bed = TestBed::new(fixture).unwrap();

    test_bed.initialize(ClientCapabilities::default()).unwrap();

    let text_document = test_bed.cursor().unwrap().text_document;
    test_bed
        .client()
        .send_request::<FoldingRangeRequest>(FoldingRangeParams {
            text_document,
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        })
        .unwrap()
        .unwrap_or_default()
}

#[test]
fn latex() {
    assert_json_snapshot!(find_foldings(
        r#"
%! main.tex
\begin{document}
    \section{Foo}
    foo
    \subsection{Bar}
    bar
    \section{Baz}
    baz
    \section{Qux}
\end{document}
|"#
    ));
}

#[test]
fn bibtex() {
    assert_json_snapshot!(find_foldings(
        r#"
%! main.bib
some junk
here

@article{foo,
    author = {bar},
    title = {baz}
}

@string{foo = "bar"}

@comment{foo,
    author = {bar},
    title = {baz}
}

@preamble{"foo"}
|"#
    ));
}
