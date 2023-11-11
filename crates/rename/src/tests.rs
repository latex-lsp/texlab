use rustc_hash::FxHashMap;

use crate::RenameParams;

fn check(input: &str) {
    let fixture = test_utils::fixture::Fixture::parse(input);

    let mut expected = FxHashMap::default();
    for spec in &fixture.documents {
        if !spec.ranges.is_empty() {
            let document = fixture.workspace.lookup(&spec.uri).unwrap();
            expected.insert(document, spec.ranges.clone());
        }
    }

    let (feature, offset) = fixture.make_params().unwrap();
    let actual = crate::rename(RenameParams { feature, offset });
    assert_eq!(actual.changes, expected);
}

#[test]
fn test_command() {
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
    )
}

#[test]
fn test_entry() {
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
    )
}

#[test]
fn test_citation() {
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
    )
}

#[test]
fn test_label() {
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
    )
}
