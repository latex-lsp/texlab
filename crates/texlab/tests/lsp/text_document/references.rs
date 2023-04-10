use lsp_types::{
    request::References, ClientCapabilities, Location, ReferenceContext, ReferenceParams,
};

use crate::fixture::TestBed;

fn sort(locations: &mut Vec<Location>) {
    locations.sort_by(|a, b| (&a.uri, a.range.start).cmp(&(&b.uri, b.range.start)));
}

fn check(fixture: &str, context: ReferenceContext) {
    let test_bed = TestBed::new(fixture).unwrap();

    test_bed.initialize(ClientCapabilities::default()).unwrap();

    let text_document_position = test_bed.cursor().unwrap();

    let mut expected = test_bed.locations().to_vec();

    let mut actual = test_bed
        .client()
        .send_request::<References>(ReferenceParams {
            text_document_position,
            context,
            partial_result_params: Default::default(),
            work_done_progress_params: Default::default(),
        })
        .unwrap()
        .unwrap_or_default();

    sort(&mut actual);
    sort(&mut expected);
    assert_eq!(actual, expected);
}

#[test]
fn entry_definition() {
    check(
        r#"
%! foo.bib
@article{foo,}
           |

%! bar.tex
\cite{foo}
      ^^^
\addbibresource{foo.bib}
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
%! foo.bib
@article{foo,}
           |
         ^^^

%! bar.tex
\cite{foo}
      ^^^
\addbibresource{foo.bib}
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
%! foo.bib
@article{foo,}

%! bar.tex
\cite{foo}
       |
      ^^^
\addbibresource{foo.bib}
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
%! foo.bib
@article{foo,}
         ^^^

%! bar.tex
\cite{foo}
       |
      ^^^
\addbibresource{foo.bib}
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
%! foo.tex
\label{foo}
        |

%! bar.tex
\ref{foo}
     ^^^
\input{foo.tex}
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
%! foo.tex
\label{foo}
        |
       ^^^

%! bar.tex
\ref{foo}
     ^^^
\input{foo.tex}
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
%! foo.tex
\label{foo}
\input{bar.tex}

%! bar.tex
\ref{foo}
      |
     ^^^

%! baz.tex
\ref{foo}
     ^^^
\input{bar.tex}
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
%! foo.tex
\label{foo}
       ^^^
\input{bar.tex}

%! bar.tex
\ref{foo}
      |
     ^^^

%! baz.tex
\ref{foo}
     ^^^
\input{bar.tex}
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
%! main.bib
@string{foo = {Foo}}
@string{bar = {Bar}}
@article{baz, author = foo}
                        |
                       ^^^
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
%! main.bib
@string{foo = {Foo}}
        ^^^
@string{bar = {Bar}}
@article{baz, author = foo}
                        |
                       ^^^
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
%! main.bib
@string{foo = {Foo}}
         |
@string{bar = {Bar}}
@article{baz, author = foo}
                       ^^^
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
%! main.bib
@string{foo = {Foo}}
         |
        ^^^
@string{bar = {Bar}}
@article{baz, author = foo}
                       ^^^
"#,
        ReferenceContext {
            include_declaration: true,
        },
    )
}
