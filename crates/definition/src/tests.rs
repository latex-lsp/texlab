use rustc_hash::FxHashSet;

use crate::{DefinitionParams, DefinitionResult};

fn check(input: &str) {
    let fixture = test_utils::fixture::Fixture::parse(input);
    let workspace = &fixture.workspace;

    let mut origin_selection_range = None;
    let mut origin_document = None;
    let mut origin_cursor = None;
    for document in &fixture.documents {
        if let Some(cursor) = document.cursor {
            origin_document = Some(document);
            origin_cursor = Some(cursor);
            origin_selection_range = document
                .ranges
                .iter()
                .find(|range| range.contains_inclusive(cursor))
                .copied();

            break;
        }
    }

    let origin_document = origin_document.unwrap();

    let mut expected = FxHashSet::default();
    for document in &fixture.documents {
        let mut ranges = document.ranges.iter();
        while let Some(target_selection_range) = ranges.next().copied() {
            let origin_selection_range = origin_selection_range.unwrap();
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

    let actual = crate::goto_definition(DefinitionParams {
        workspace,
        document: workspace.lookup(&origin_document.uri).unwrap(),
        offset: origin_cursor.unwrap(),
    });

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
