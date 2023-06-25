use std::borrow::Cow;

use test_utils::fixture::Fixture;

use crate::{
    types::{CitationError, Diagnostic, DiagnosticData, LabelError, SyntaxError},
    DiagnosticBuilder, DiagnosticManager, DiagnosticSource,
};

fn check(input: &str, expected_data: &[DiagnosticData]) {
    let fixture = Fixture::parse(input);
    let mut manager = DiagnosticManager::default();

    let mut expected = DiagnosticBuilder::default();
    let mut expected_data = expected_data.iter();
    for document in &fixture.documents {
        let diagnostics = document.ranges.iter().copied().map(|range| {
            let data = expected_data.next().unwrap().clone();
            Cow::Owned(Diagnostic { range, data })
        });

        expected.push_many(&document.uri, diagnostics);
    }

    for document in fixture.workspace.iter() {
        manager.update(&fixture.workspace, &document);
    }

    let mut actual = DiagnosticBuilder::default();
    manager.publish(&fixture.workspace, &mut actual);

    for diagnostics in actual.inner.values_mut() {
        diagnostics.sort_by_key(|diag| (diag.range.start(), diag.range.len()));
    }

    assert_eq!(actual, expected);
}

#[test]
fn test_bib_entry_missing_l_delim() {
    check(
        r#"
%! main.bib
@article
        !
"#,
        &[DiagnosticData::Syntax(SyntaxError::ExpectingLCurly)],
    )
}

#[test]
fn test_bib_entry_missing_r_delim() {
    check(
        r#"
%! main.bib
@article{foo,
              !

%! main.tex
\bibliography{main}
\cite{foo}
"#,
        &[DiagnosticData::Syntax(SyntaxError::ExpectingRCurly)],
    )
}

#[test]
fn test_bib_entry_missing_name() {
    check(
        r#"
%! main.bib
@article{
         !"#,
        &[DiagnosticData::Syntax(SyntaxError::ExpectingKey)],
    )
}

#[test]
fn test_bib_field_missing_eq() {
    check(
        r#"
%! main.bib
@article{foo,
    field
         !
}

%! main.tex
\bibliography{main}
\cite{foo}
"#,
        &[DiagnosticData::Syntax(SyntaxError::ExpectingEq)],
    )
}

#[test]
fn test_bib_field_missing_value() {
    check(
        r#"
%! main.bib
@article{foo,
    field =
           !
}

%! main.tex
\bibliography{main}
\cite{foo}
"#,
        &[DiagnosticData::Syntax(SyntaxError::ExpectingFieldValue)],
    )
}

#[test]
fn test_tex_unmatched_braces() {
    check(
        r#"
%! main.tex
} 
^
{
  !
"#,
        &[
            DiagnosticData::Syntax(SyntaxError::UnexpectedRCurly),
            DiagnosticData::Syntax(SyntaxError::RCurlyInserted),
        ],
    )
}

#[test]
fn test_tex_environment_mismatched() {
    check(
        r#"
%! main.tex
\begin{foo}
       ^^^
\end{bar}
"#,
        &[DiagnosticData::Syntax(SyntaxError::MismatchedEnvironment)],
    )
}

#[test]
fn test_label_unused() {
    check(
        r#"
%! main.tex
\label{foo}
       ^^^
\label{bar}\ref{bar}
"#,
        &[DiagnosticData::Label(LabelError::Unused)],
    )
}

#[test]
fn test_label_undefined() {
    check(
        r#"
%! main.tex
\ref{foo}
     ^^^
"#,
        &[DiagnosticData::Label(LabelError::Undefined)],
    )
}

#[test]
fn test_citation_undefined() {
    check(
        r#"
%! main.tex
\cite{foo}
      ^^^
"#,
        &[DiagnosticData::Citation(CitationError::Undefined)],
    )
}

#[test]
fn test_citation_unused() {
    check(
        r#"
%! main.bib
@article{foo,}
         ^^^
"#,
        &[DiagnosticData::Citation(CitationError::Unused)],
    )
}
