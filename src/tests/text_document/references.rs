use assert_unordered::assert_eq_unordered;
use lsp_types::{
    request::References, ClientCapabilities, Location, ReferenceContext, ReferenceParams,
};

use crate::tests::{client::Client, fixture};

fn check(fixture: &str, context: ReferenceContext) {
    let mut client = Client::spawn();
    client.initialize(ClientCapabilities::default(), None);

    let fixture = fixture::parse(fixture);
    for file in fixture.files {
        client.open(file.name, file.lang, file.text);
    }

    let mut expected_locations = Vec::new();
    for ranges in fixture.ranges.values() {
        expected_locations.push(Location::new(client.uri(ranges[&1].name), ranges[&1].range));
    }

    let actual_locations = client
        .request::<References>(ReferenceParams {
            text_document_position: fixture.cursor.unwrap().into_params(&client),
            context,
            partial_result_params: Default::default(),
            work_done_progress_params: Default::default(),
        })
        .unwrap()
        .unwrap_or_default();

    client.shutdown();

    assert_eq_unordered!(actual_locations, expected_locations);
}

#[test]
fn entry_definition() {
    check(
        r#"
%BIB foo.bib
%SRC @article{foo,}
%CUR            ^

%TEX bar.tex
%SRC \cite{foo}
%1.1       ^^^
%SRC \addbibresource{foo.bib}
"#,
        ReferenceContext {
            include_declaration: false,
        },
    )
}

#[test]
fn entry_definition_include_decl() {
    check(
        r#"
%BIB foo.bib
%SRC @article{foo,}
%CUR            ^
%2.1          ^^^

%TEX bar.tex
%SRC \cite{foo}
%1.1       ^^^
%SRC \addbibresource{foo.bib}
"#,
        ReferenceContext {
            include_declaration: true,
        },
    )
}

#[test]
fn entry_reference() {
    check(
        r#"
%BIB foo.bib
%SRC @article{foo,}

%TEX bar.tex
%SRC \cite{foo}
%CUR        ^
%1.1       ^^^
%SRC \addbibresource{foo.bib}
"#,
        ReferenceContext {
            include_declaration: false,
        },
    )
}

#[test]
fn entry_reference_include_decl() {
    check(
        r#"
%BIB foo.bib
%SRC @article{foo,}
%2.1          ^^^

%TEX bar.tex
%SRC \cite{foo}
%CUR        ^
%1.1       ^^^
%SRC \addbibresource{foo.bib}
"#,
        ReferenceContext {
            include_declaration: true,
        },
    )
}

#[test]
fn label_definition() {
    check(
        r#"
%TEX foo.tex
%SRC \label{foo}
%CUR         ^

%TEX bar.tex
%SRC \ref{foo}
%1.1      ^^^
%SRC \input{foo.tex}
"#,
        ReferenceContext {
            include_declaration: false,
        },
    )
}

#[test]
fn label_definition_include_decl() {
    check(
        r#"
%TEX foo.tex
%SRC \label{foo}
%CUR         ^
%2.1        ^^^

%TEX bar.tex
%SRC \ref{foo}
%1.1      ^^^
%SRC \input{foo.tex}
"#,
        ReferenceContext {
            include_declaration: true,
        },
    )
}

#[test]
fn label_reference() {
    check(
        r#"
%TEX foo.tex
%SRC \label{foo}
%SRC \input{bar.tex}

%TEX bar.tex
%SRC \ref{foo}
%CUR       ^
%1.1      ^^^

%TEX baz.tex
%SRC \ref{foo}
%2.1      ^^^
%SRC \input{bar.tex}
"#,
        ReferenceContext {
            include_declaration: false,
        },
    )
}

#[test]
fn label_reference_include_decl() {
    check(
        r#"
%TEX foo.tex
%SRC \label{foo}
%3.1        ^^^
%SRC \input{bar.tex}

%TEX bar.tex
%SRC \ref{foo}
%CUR       ^
%1.1      ^^^

%TEX baz.tex
%SRC \ref{foo}
%2.1      ^^^
%SRC \input{bar.tex}
"#,
        ReferenceContext {
            include_declaration: true,
        },
    )
}

#[test]
fn string_reference() {
    check(
        r#"
%BIB main.bib
%SRC @string{foo = {Foo}}
%SRC @string{bar = {Bar}}
%SRC @article{baz, author = foo}
%CUR                         ^
%1.1                        ^^^
"#,
        ReferenceContext {
            include_declaration: false,
        },
    )
}

#[test]
fn string_reference_include_decl() {
    check(
        r#"
%BIB main.bib
%SRC @string{foo = {Foo}}
%2.1         ^^^
%SRC @string{bar = {Bar}}
%SRC @article{baz, author = foo}
%CUR                         ^
%1.1                        ^^^
"#,
        ReferenceContext {
            include_declaration: true,
        },
    )
}

#[test]
fn string_definition() {
    check(
        r#"
%BIB main.bib
%SRC @string{foo = {Foo}}
%CUR          ^
%SRC @string{bar = {Bar}}
%SRC @article{baz, author = foo}
%1.1                        ^^^
"#,
        ReferenceContext {
            include_declaration: false,
        },
    )
}

#[test]
fn string_definition_include_decl() {
    check(
        r#"
%BIB main.bib
%SRC @string{foo = {Foo}}
%CUR          ^
%2.1         ^^^
%SRC @string{bar = {Bar}}
%SRC @article{baz, author = foo}
%1.1                        ^^^
"#,
        ReferenceContext {
            include_declaration: true,
        },
    )
}
