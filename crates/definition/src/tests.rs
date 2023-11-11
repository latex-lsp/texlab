use rowan::TextRange;
use rustc_hash::FxHashSet;

use crate::{DefinitionParams, DefinitionResult};

fn check(input: &str) {
    let fixture = test_utils::fixture::Fixture::parse(input);
    let (feature, offset) = fixture.make_params().unwrap();
    let origin_document = feature.document;
    let origin_selection_range = fixture
        .locations()
        .filter(|location| location.document == origin_document)
        .find(|location| location.range.contains_inclusive(offset))
        .map_or_else(|| TextRange::default(), |location| location.range);

    let mut expected = FxHashSet::default();
    for document in &fixture.documents {
        let mut ranges = document.ranges.iter();
        while let Some(target_selection_range) = ranges.next().copied() {
            if (&origin_document.uri, origin_selection_range)
                != (&document.uri, target_selection_range)
            {
                expected.insert(DefinitionResult {
                    origin_selection_range,
                    target: fixture.workspace.lookup(&document.uri).unwrap(),
                    target_range: *ranges.next().unwrap(),
                    target_selection_range,
                });
            }
        }
    }

    let actual = crate::goto_definition(DefinitionParams { feature, offset });

    assert_eq!(actual, expected);
}

#[test]
fn test_command_definition() {
    check(
        r#"
%! main.tex
\DeclareMathOperator{\foo}{foo}
                     ^^^^
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
\foo
  |
^^^^"#,
    )
}

#[test]
fn test_document() {
    check(
        r#"
%! foo.tex
\addbibresource{baz.bib}
                  |
                ^^^^^^^

%! bar.bib
@article{foo, bar = {baz}}

%! baz.bib
@article{foo, bar = {baz}}
!
!"#,
    )
}

#[test]
fn test_entry() {
    check(
        r#"
%! foo.tex
\addbibresource{baz.bib}
\cite{foo}
      |
      ^^^

%! bar.bib
@article{foo, bar = {baz}}

%! baz.bib
@article{foo, bar = {baz}}
         ^^^
^^^^^^^^^^^^^^^^^^^^^^^^^^"#,
    )
}

#[test]
fn test_string_simple() {
    check(
        r#"
%! main.bib
@string{foo = {bar}}
        ^^^
^^^^^^^^^^^^^^^^^^^^
@article{bar, author = foo}
                        |
                       ^^^"#,
    )
}

#[test]
fn test_string_join() {
    check(
        r#"
%! main.bib
@string{foo = {bar}}
        ^^^
^^^^^^^^^^^^^^^^^^^^
@article{bar, author = foo # "bar"}
                        |
                       ^^^"#,
    )
}

#[test]
fn test_string_field() {
    check(
        r#"
%! main.bib
@string{foo = {bar}}
@article{bar, author = foo # "bar"}
                |"#,
    )
}
