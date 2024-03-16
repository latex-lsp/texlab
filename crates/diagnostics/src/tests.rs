use expect_test::{expect, Expect};
use itertools::Itertools;

fn check(input: &str, expect: Expect) {
    let fixture = test_utils::fixture::Fixture::parse(input);
    let mut manager = crate::Manager::default();

    for document in fixture.workspace.iter() {
        manager.update_syntax(&fixture.workspace, &document);
    }

    let results = manager.get(&fixture.workspace);
    let results = results
        .iter()
        .filter(|(_, diags)| !diags.is_empty())
        .sorted_by(|(uri1, _), (uri2, _)| uri1.cmp(&uri2))
        .map(|(uri, diags)| (uri.as_str(), diags))
        .collect_vec();

    expect.assert_debug_eq(&results);
}

#[test]
fn test_bib_entry_missing_l_delim() {
    check(
        r#"
%! main.bib
@article
        !
"#,
        expect![[r#"
            [
                (
                    "file:///texlab/main.bib",
                    [
                        Bib(
                            8..8,
                            ExpectingLCurly,
                        ),
                    ],
                ),
            ]
        "#]],
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
        expect![[r#"
            [
                (
                    "file:///texlab/main.bib",
                    [
                        Bib(
                            14..14,
                            ExpectingRCurly,
                        ),
                    ],
                ),
            ]
        "#]],
    )
}

#[test]
fn test_bib_entry_missing_name() {
    check(
        r#"
%! main.bib
@article{
         !"#,
        expect![[r#"
            [
                (
                    "file:///texlab/main.bib",
                    [
                        Bib(
                            9..9,
                            ExpectingKey,
                        ),
                    ],
                ),
            ]
        "#]],
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
        expect![[r#"
            [
                (
                    "file:///texlab/main.bib",
                    [
                        Bib(
                            23..23,
                            ExpectingEq,
                        ),
                    ],
                ),
            ]
        "#]],
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
        expect![[r#"
            [
                (
                    "file:///texlab/main.bib",
                    [
                        Bib(
                            25..25,
                            ExpectingFieldValue,
                        ),
                    ],
                ),
            ]
        "#]],
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
        expect![[r#"
            [
                (
                    "file:///texlab/main.tex",
                    [
                        Tex(
                            0..1,
                            UnexpectedRCurly,
                        ),
                        Tex(
                            4..4,
                            ExpectingRCurly,
                        ),
                    ],
                ),
            ]
        "#]],
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
        expect![[r#"
            [
                (
                    "file:///texlab/main.tex",
                    [
                        Tex(
                            7..10,
                            MismatchedEnvironment,
                        ),
                    ],
                ),
            ]
        "#]],
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
        expect![[r#"
            [
                (
                    "file:///texlab/main.tex",
                    [
                        Tex(
                            7..10,
                            UnusedLabel,
                        ),
                    ],
                ),
            ]
        "#]],
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
        expect![[r#"
            [
                (
                    "file:///texlab/main.tex",
                    [
                        Tex(
                            5..8,
                            UndefinedLabel,
                        ),
                    ],
                ),
            ]
        "#]],
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
        expect![[r#"
            [
                (
                    "file:///texlab/main.tex",
                    [
                        Tex(
                            6..9,
                            UndefinedCitation,
                        ),
                    ],
                ),
            ]
        "#]],
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
        expect![[r#"
            [
                (
                    "file:///texlab/main.bib",
                    [
                        Bib(
                            9..12,
                            UnusedEntry,
                        ),
                    ],
                ),
            ]
        "#]],
    )
}
