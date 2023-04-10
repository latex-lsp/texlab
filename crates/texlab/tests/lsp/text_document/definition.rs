use itertools::Itertools;
use lsp_types::{
    request::GotoDefinition, ClientCapabilities, GotoDefinitionParams, GotoDefinitionResponse,
    LocationLink,
};

use crate::fixture::TestBed;

fn check(fixture: &str) {
    let test_bed = TestBed::new(fixture).unwrap();
    test_bed.initialize(ClientCapabilities::default()).unwrap();

    let text_document_position_params = test_bed.cursor().unwrap();
    let cursor = text_document_position_params.position;

    let origin_selection = test_bed
        .locations()
        .iter()
        .filter(|location| location.uri == text_document_position_params.text_document.uri)
        .find(|location| cursor >= location.range.start && cursor <= location.range.end);

    let mut expected_links: Vec<_> = test_bed
        .locations()
        .iter()
        .filter(|location| Some(*location) != origin_selection)
        .batching(|it| {
            let target_selection_range = it.next()?.range;
            let target = it.next()?;
            Some(LocationLink {
                origin_selection_range: origin_selection.map(|sel| sel.range),
                target_uri: target.uri.clone(),
                target_range: target.range,
                target_selection_range,
            })
        })
        .collect();

    let mut actual_links = match test_bed
        .client()
        .send_request::<GotoDefinition>(GotoDefinitionParams {
            text_document_position_params,
            partial_result_params: Default::default(),
            work_done_progress_params: Default::default(),
        })
        .unwrap()
    {
        Some(GotoDefinitionResponse::Link(links)) => links,
        Some(GotoDefinitionResponse::Array(_)) => unreachable!(),
        Some(GotoDefinitionResponse::Scalar(_)) => unreachable!(),
        None => Vec::new(),
    };

    sort_links(&mut actual_links);
    sort_links(&mut expected_links);
    assert_eq!(actual_links, expected_links);
}

fn sort_links(links: &mut Vec<LocationLink>) {
    links.sort_by(|a, b| {
        let left = (&a.target_uri, a.target_range.start);
        let right = (&b.target_uri, b.target_range.start);
        left.cmp(&right)
    });
}

#[test]
fn command_definition() {
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
fn document() {
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
fn entry() {
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
fn string_simple() {
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
fn string_join() {
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
fn string_field() {
    check(
        r#"
%! main.bib
@string{foo = {bar}}
@article{bar, author = foo # "bar"}
                |"#,
    )
}
