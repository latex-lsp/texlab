use expect_test::{expect, Expect};

use crate::HoverParams;

fn check(input: &str, expect: Expect) {
    let fixture = test_utils::fixture::Fixture::parse(input);
    let (feature, offset) = fixture.make_params().unwrap();
    let params = HoverParams { feature, offset };
    let data = crate::find(&params).map(|hover| {
        assert_eq!(fixture.documents[0].ranges[0], hover.range);
        hover.data
    });

    expect.assert_debug_eq(&data);
}

#[test]
fn test_smoke() {
    check(
        r#"
%! main.tex

|"#,
        expect![[r#"
            None
        "#]],
    );
}

#[test]
fn test_latex_citation() {
    check(
        r#"
%! main.tex
\addbibresource{main.bib}
\cite{foo}
       |
      ^^^
%! main.bib
@article{foo, author = {Foo Bar}, title = {Baz Qux}, year = 1337}"#,
        expect![[r#"
            Some(
                Citation(
                    "F. Bar: \"Baz Qux\". (1337).",
                ),
            )
        "#]],
    );
}

#[test]
fn test_bibtex_entry_key() {
    check(
        r#"
%! main.bib
@article{foo, author = {Foo Bar}, title = {Baz Qux}, year = 1337}
          |
         ^^^

%! main.tex
\addbibresource{main.bib}
\cite{foo}"#,
        expect![[r#"
            Some(
                Citation(
                    "F. Bar: \"Baz Qux\". (1337).",
                ),
            )
        "#]],
    );
}

#[test]
fn test_bibtex_entry_key_empty() {
    check(
        r#"
%! main.bib
@foo{bar,}
      |"#,
        expect![[r#"
            None
        "#]],
    );
}

#[test]
fn test_bibtex_entry_type_known() {
    check(
        r#"
%! main.bib
@article{foo,}
    |
^^^^^^^^"#,
        expect![[r#"
            Some(
                EntryType(
                    BibtexEntryType {
                        name: "@article",
                        category: Article,
                        documentation: Some(
                            "An article in a journal, magazine, newspaper, or other periodical which forms a \n self-contained unit with its own title. The title of the periodical is given in the \n journaltitle field. If the issue has its own title in addition to the main title of \n the periodical, it goes in the issuetitle field. Note that editor and related \n fields refer to the journal while translator and related fields refer to the article.\n\nRequired fields: `author`, `title`, `journaltitle`, `year/date`",
                        ),
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn test_bibtex_entry_type_unknown() {
    check(
        r#"
%! main.bib
@foo{bar,}
  |"#,
        expect![[r#"
            None
        "#]],
    );
}

#[test]
fn test_bibtex_field_known() {
    check(
        r#"
%! main.bib
@article{foo, author = bar}
               |
              ^^^^^^"#,
        expect![[r#"
            Some(
                FieldType(
                    BibtexFieldType {
                        name: "author",
                        documentation: "The author(s) of the `title`.",
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn test_bibtex_field_unknown() {
    check(
        r#"
%! main.bib
@article{foo, bar = baz}
               |"#,
        expect![[r#"
            None
        "#]],
    );
}

#[test]
fn test_bibtex_string_ref() {
    check(
        r#"
%! main.bib
@string{foo = "Foo"}
@string{bar = "Bar"}
@article{baz, author = bar}
                        |
                       ^^^"#,
        expect![[r#"
            Some(
                StringRef(
                    "Bar",
                ),
            )
        "#]],
    );
}

#[test]
fn test_bibtex_value() {
    check(
        r#"
%! main.bib
@string{foo = "Foo"}
@string{bar = "Bar"}
@article{baz, author = bar}
                     |"#,
        expect![[r#"
            None
        "#]],
    );
}

#[test]
fn test_latex_package_known() {
    check(
        r#"
%! main.tex
\usepackage{amsmath}
             |
            ^^^^^^^"#,
        expect![[r#"
            Some(
                Package(
                    "The package provides the principal packages in the AMS-LaTeX distribution. It adapts for use in LaTeX most of the mathematical features found in AMS-TeX; it is highly recommended as an adjunct to serious mathematical typesetting in LaTeX. When amsmath is loaded, AMS-LaTeX packages amsbsy (for bold symbols), amsopn (for operator names) and amstext (for text embedded in mathematics) are also loaded. amsmath is part of the LaTeX required distribution; however, several contributed packages add still further to its appeal; examples are empheq, which provides functions for decorating and highlighting mathematics, and ntheorem, for specifying theorem (and similar) definitions.",
                ),
            )
        "#]],
    );
}

#[test]
fn test_latex_class_unknown() {
    check(
        r#"
%! main.tex
\documentclass{abcdefghijklmnop}
                    |"#,
        expect![[r#"
            None
        "#]],
    );
}

#[test]
fn test_latex_label_section() {
    check(
        r#"
%! main.tex
\section{Foo}
\label{sec:foo}
         |
       ^^^^^^^"#,
        expect![[r#"
            Some(
                Label(
                    RenderedLabel {
                        range: 0..29,
                        number: None,
                        object: Section {
                            prefix: "Section",
                            text: "Foo",
                        },
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn test_latex_label_theorem_child_file() {
    check(
        r#"
%! main.tex
\documentclass{article}
\newtheorem{lemma}{Lemma}
\include{child}
\ref{thm:foo}
        |
     ^^^^^^^

%! child.tex
\begin{lemma}\label{thm:foo}
    1 + 1 = 2
\end{lemma}"#,
        expect![[r#"
            Some(
                Label(
                    RenderedLabel {
                        range: 0..54,
                        number: None,
                        object: Theorem {
                            kind: "Lemma",
                            description: None,
                        },
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn test_latex_label_theorem_child_file_mumber() {
    check(
        r#"
%! main.tex
\documentclass{article}
\newtheorem{lemma}{Lemma}
\include{child}
\ref{thm:foo}
        |
     ^^^^^^^

%! child.tex
\begin{lemma}[Foo]\label{thm:foo}
    1 + 1 = 2
\end{lemma}

%! child.aux
\newlabel{thm:foo}{{1}{1}{Foo}{lemma.1}{}}"#,
        expect![[r#"
            Some(
                Label(
                    RenderedLabel {
                        range: 0..59,
                        number: Some(
                            "1",
                        ),
                        object: Theorem {
                            kind: "Lemma",
                            description: Some(
                                "Foo",
                            ),
                        },
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn test_latex_label_ntheorem() {
    check(
        r#"
%! main.tex
\newtheorem{theorem}[theoremcounter]{Theorem}
\begin{theorem}%
\label{thm:test}
\end{theorem}
\ref{thm:test}
        |
     ^^^^^^^^

%! main.aux
\newlabel{thm:test}{{1.{1}}{1}}"#,
        expect![[r#"
            Some(
                Label(
                    RenderedLabel {
                        range: 46..93,
                        number: Some(
                            "1.1",
                        ),
                        object: Theorem {
                            kind: "Theorem",
                            description: None,
                        },
                    },
                ),
            )
        "#]],
    );
}
