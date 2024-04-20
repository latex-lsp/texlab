use std::collections::HashSet;

use crate::ReferenceParams;

fn check(fixture: &str, include_declaration: bool) {
    let fixture = test_utils::fixture::Fixture::parse(fixture);
    let (feature, offset) = fixture.make_params().unwrap();

    let expected = fixture.locations().collect::<HashSet<_>>();
    let actual = crate::find_all(&ReferenceParams {
        feature,
        offset,
        include_declaration,
    })
    .into_iter()
    .collect::<HashSet<_>>();

    assert_eq!(actual, expected);
}

#[test]
fn test_entry_definition() {
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
        false,
    );
}

#[test]
fn test_entry_definition_include_decl() {
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
        true,
    );
}

#[test]
fn test_entry_reference() {
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
        false,
    );
}

#[test]
fn test_entry_reference_include_decl() {
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
        true,
    );
}

#[test]
fn test_label_definition() {
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
        false,
    );
}

#[test]
fn test_label_definition_include_decl() {
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
        true,
    );
}

#[test]
fn test_label_reference() {
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
        false,
    );
}

#[test]
fn test_label_reference_include_decl() {
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
        true,
    );
}

#[test]
fn test_string_reference() {
    check(
        r#"
%! main.bib
@string{foo = {Foo}}
@string{bar = {Bar}}
@article{baz, author = foo}
                        |
                       ^^^
"#,
        false,
    );
}

#[test]
fn test_string_reference_include_decl() {
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
        true,
    );
}

#[test]
fn test_string_definition() {
    check(
        r#"
%! main.bib
@string{foo = {Foo}}
         |
@string{bar = {Bar}}
@article{baz, author = foo}
                       ^^^
"#,
        false,
    );
}

#[test]
fn test_string_definition_include_decl() {
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
        true,
    );
}

#[test]
fn test_new_command_definition() {
    check(
        r#"
%! main.tex
\foo
  |
 ^^^

\newcommand{\foo}{foo}
"#,
        false,
    );
}

#[test]
fn test_new_command_definition_include_decl() {
    check(
        r#"
%! main.tex
\foo
  |
 ^^^

\newcommand{\foo}{foo}
             ^^^
"#,
        true,
    );
}
