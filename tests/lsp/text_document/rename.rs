use std::collections::HashMap;

use lsp_types::{request::Rename, ClientCapabilities, RenameParams, TextEdit, Url, WorkspaceEdit};

use crate::fixture::TestBed;

fn check(fixture: &str, new_name: &str) {
    let test_bed = TestBed::new(fixture).unwrap();

    test_bed.initialize(ClientCapabilities::default()).unwrap();

    let mut expected_changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();
    for location in test_bed.locations() {
        expected_changes
            .entry(location.uri.clone())
            .or_default()
            .push(TextEdit::new(location.range, new_name.to_string()));
    }

    let text_document_position = test_bed.cursor().unwrap();
    let actual_edit = test_bed
        .client()
        .send_request::<Rename>(RenameParams {
            text_document_position,
            new_name: new_name.to_string(),
            work_done_progress_params: Default::default(),
        })
        .unwrap()
        .unwrap_or_default();

    assert_eq!(actual_edit, WorkspaceEdit::new(expected_changes));
}

#[test]
fn command() {
    check(
        r#"
%! foo.tex
\baz
  |
 ^^^
\include{bar.tex}

%! bar.tex
\baz
 ^^^
"#,
        "qux",
    )
}

#[test]
fn entry() {
    check(
        r#"
%! main.bib
@article{foo, bar = baz}
         |
         ^^^

%! main.tex
\addbibresource{main.bib}
\cite{foo}
      ^^^
"#,
        "qux",
    )
}

#[test]
fn citation() {
    check(
        r#"
%! main.bib
@article{foo, bar = baz}
         ^^^

%! main.tex
\addbibresource{main.bib}
\cite{foo}
       |
      ^^^
"#,
        "qux",
    )
}

#[test]
fn label() {
    check(
        r#"
%! foo.tex
\label{foo}\include{bar}
       |
       ^^^

%! bar.tex
\ref{foo}
     ^^^

%! baz.tex
\ref{foo}
"#,
        "bar",
    )
}
