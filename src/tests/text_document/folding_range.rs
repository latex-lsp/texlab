use anyhow::Result;
use assert_unordered::assert_eq_unordered;
use lsp_types::{
    request::FoldingRangeRequest, ClientCapabilities, FoldingRange, FoldingRangeKind,
    FoldingRangeParams, TextDocumentIdentifier,
};

use crate::tests::{client::Client, fixture};

fn check(fixture: &str, expected_ranges: Vec<(u32, u32, u32, u32)>) -> Result<()> {
    let mut client = Client::spawn()?;
    client.initialize(ClientCapabilities::default(), None)?;

    let fixture = fixture::parse(fixture);
    for file in fixture.files {
        client.open(file.name, file.lang, file.text)?;
    }

    let actual_foldings = client
        .request::<FoldingRangeRequest>(FoldingRangeParams {
            text_document: TextDocumentIdentifier::new(client.uri(fixture.cursor.unwrap().name)?),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        })?
        .unwrap_or_default();

    client.shutdown()?;

    let expected_foldings = expected_ranges
        .into_iter()
        .map(
            |(start_line, start_character, end_line, end_character)| FoldingRange {
                start_line,
                start_character: Some(start_character),
                end_line,
                end_character: Some(end_character),
                kind: Some(FoldingRangeKind::Region),
            },
        )
        .collect();

    assert_eq_unordered!(actual_foldings, expected_foldings);
    Ok(())
}

#[test]
fn latex() -> Result<()> {
    check(
        r#"
%TEX main.tex
%SRC \begin{document}
%SRC     \section{Foo}
%SRC     foo
%SRC     \subsection{Bar}
%SRC     bar
%SRC     \section{Baz}
%SRC     baz
%SRC     \section{Qux}
%SRC \end{document}
%CUR ^
"#,
        vec![
            (0, 0, 8, 14),
            (1, 4, 4, 7),
            (3, 4, 4, 7),
            (5, 4, 6, 7),
            (7, 4, 7, 17),
        ],
    )
}

#[test]
fn bibtex() -> Result<()> {
    check(
        r#"
%BIB main.bib
%SRC some junk
%SRC here
%SRC 
%SRC @article{foo,
%SRC     author = {bar},
%SRC     title = {baz}
%SRC }
%SRC 
%SRC @string{foo = "bar"}
%SRC 
%SRC @comment{foo,
%SRC     author = {bar},
%SRC     title = {baz}
%SRC }
%SRC 
%SRC @preamble{"foo"}
%CUR ^
"#,
        vec![(3, 0, 6, 1), (8, 0, 8, 20), (15, 0, 15, 16)],
    )
}
