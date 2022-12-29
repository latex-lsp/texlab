use std::collections::HashMap;

use lsp_types::{request::Rename, ClientCapabilities, RenameParams, TextEdit, Url, WorkspaceEdit};

use crate::tests::{client::Client, fixture};

fn check(fixture: &str, new_name: &str) {
    let mut client = Client::spawn();
    client.initialize(ClientCapabilities::default(), None);

    let fixture = fixture::parse(fixture);
    for file in fixture.files {
        client.open(file.name, file.lang, file.text);
    }

    let mut expected_changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();
    for ranges in fixture.ranges.values() {
        expected_changes
            .entry(client.uri(ranges[&1].name))
            .or_default()
            .push(TextEdit::new(ranges[&1].range, new_name.to_string()));
    }

    let actual_edit = client
        .request::<Rename>(RenameParams {
            text_document_position: fixture.cursor.unwrap().into_params(&client),
            new_name: new_name.to_string(),
            work_done_progress_params: Default::default(),
        })
        .unwrap()
        .unwrap_or_default();

    client.shutdown();

    assert_eq!(actual_edit, WorkspaceEdit::new(expected_changes));
}

#[test]
fn command() {
    check(
        r#"
%TEX foo.tex
%SRC \baz
%CUR   ^
%1.1  ^^^
%SRC \include{bar.tex}

%TEX bar.tex
%SRC \baz
%2.1  ^^^
"#,
        "qux",
    )
}

#[test]
fn entry() {
    check(
        r#"
%BIB main.bib
%SRC @article{foo, bar = baz}
%CUR          ^
%1.1          ^^^

%TEX main.tex
%SRC \addbibresource{main.bib}
%SRC \cite{foo}
%2.1       ^^^
"#,
        "qux",
    )
}

#[test]
fn citation() {
    check(
        r#"
%BIB main.bib
%SRC @article{foo, bar = baz}
%1.1          ^^^

%TEX main.tex
%SRC \addbibresource{main.bib}
%SRC \cite{foo}
%CUR        ^
%2.1       ^^^
"#,
        "qux",
    )
}

#[test]
fn label() {
    check(
        r#"
%TEX foo.tex
%SRC \label{foo}\include{bar}
%CUR        ^
%1.1        ^^^

%TEX bar.tex
%SRC \ref{foo}
%2.1      ^^^

%TEX baz.tex
%SRC \ref{foo}
"#,
        "bar",
    )
}
