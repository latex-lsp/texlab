use expect_test::{expect, Expect};

fn check(input: &str, expect: Expect) {
    let fixture = test_utils::fixture::Fixture::parse(input);
    let document = fixture.make_params().unwrap().0.document;
    let data = crate::find_all(document);
    expect.assert_debug_eq(&data);
}

#[test]
fn test_latex() {
    check(
        r#"
%! main.tex
\begin{document}
    \section{Foo}
    foo
    \subsection{Bar}
    bar
    \section{Baz}
    baz
    \section{Qux}
\end{document}
|"#,
        expect![[r#"
            [
                FoldingRange {
                    range: 16..116,
                    kind: Environment,
                },
                FoldingRange {
                    range: 34..76,
                    kind: Section,
                },
                FoldingRange {
                    range: 63..76,
                    kind: Section,
                },
                FoldingRange {
                    range: 89..102,
                    kind: Section,
                },
                FoldingRange {
                    range: 115..116,
                    kind: Section,
                },
            ]
        "#]],
    );
}

#[test]
fn test_bibtex() {
    check(
        r#"
%! main.bib
some junk
here

@article{foo,
    author = {bar},
    title = {baz}
}

@string{foo = "bar"}

@comment{foo,
    author = {bar},
    title = {baz}
}

@preamble{"foo"}
|"#,
        expect![[r#"
            [
                FoldingRange {
                    range: 28..68,
                    kind: Entry,
                },
                FoldingRange {
                    range: 82..90,
                    kind: Entry,
                },
            ]
        "#]],
    );
}
