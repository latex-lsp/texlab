use base_db::FeatureParams;
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

    let (document, offset) = fixture
        .documents
        .iter()
        .find_map(|spec| Some((fixture.workspace.lookup(&spec.uri)?, spec.cursor?)))
        .unwrap();

    let inner = FeatureParams::new(&fixture.workspace, document);
    let params = RenameParams { inner, offset };
    let actual = crate::rename(&params);
    assert_eq!(actual.changes, expected);
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
    )
}
