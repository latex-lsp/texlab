use expect_test::{expect, Expect};
use test_utils::fixture::Fixture;

use crate::workspace_symbols;

static FIXTURE: &str = r#"
%! main.tex
\documentclass{article}
\usepackage{caption}
\usepackage{amsmath}
\usepackage{amsthm}

\begin{document}

\section{Foo}\label{sec:foo}

\begin{equation}\label{eq:foo}
    Foo
\end{equation}

\section{Bar}\label{sec:bar}

\begin{figure}
    Bar
    \caption{Bar}
    \label{fig:bar}
\end{figure}

\section{Baz}\label{sec:baz}

\begin{enumerate}
    \item\label{itm:foo} Foo
    \item\label{itm:bar} Bar
    \item\label{itm:baz} Baz
\end{enumerate}

\section{Qux}\label{sec:qux}

\newtheorem{lemma}{Lemma}

\begin{lemma}[Qux]\label{thm:qux}
    Qux
\end{lemma}

\end{document}

%! main.aux
\relax
\@writefile{lof}{\contentsline {figure}{\numberline {1}{\ignorespaces Bar\relax }}{1}\protected@file@percent }
\providecommand*\caption@xref[2]{\@setref\relax\@undefined{#1}}
\newlabel{fig:bar}{{1}{1}}
\@writefile{toc}{\contentsline {section}{\numberline {1}Foo}{1}\protected@file@percent }
\newlabel{sec:foo}{{1}{1}}
\newlabel{eq:foo}{{1}{1}}
\@writefile{toc}{\contentsline {section}{\numberline {2}Bar}{1}\protected@file@percent }
\newlabel{sec:bar}{{2}{1}}
\@writefile{toc}{\contentsline {section}{\numberline {3}Baz}{1}\protected@file@percent }
\newlabel{sec:baz}{{3}{1}}
\newlabel{itm:foo}{{1}{1}}
\newlabel{itm:bar}{{2}{1}}
\newlabel{itm:baz}{{3}{1}}
\@writefile{toc}{\contentsline {section}{\numberline {4}Qux}{1}\protected@file@percent }
\newlabel{sec:qux}{{4}{1}}
\newlabel{thm:qux}{{1}{1}}

%! main.bib
@article{foo,}

@string{bar = "bar"}"#;

fn check(query: &str, expect: Expect) {
    let fixture = Fixture::parse(FIXTURE);
    let symbols = workspace_symbols(&fixture.workspace, query);
    expect.assert_debug_eq(&symbols);
}

#[test]
fn filter_type_section() {
    check(
        "section",
        expect![[r#"
        [
            SymbolLocation {
                document: Document(
                    "file:///texlab/main.tex",
                ),
                symbol: Symbol {
                    name: "1 Foo",
                    kind: Section,
                    label: Some(
                        Span(
                            "sec:foo",
                            118..133,
                        ),
                    ),
                    full_range: 105..188,
                    selection_range: 118..133,
                    children: [],
                },
            },
            SymbolLocation {
                document: Document(
                    "file:///texlab/main.tex",
                ),
                symbol: Symbol {
                    name: "2 Bar",
                    kind: Section,
                    label: Some(
                        Span(
                            "sec:bar",
                            203..218,
                        ),
                    ),
                    full_range: 190..293,
                    selection_range: 203..218,
                    children: [],
                },
            },
            SymbolLocation {
                document: Document(
                    "file:///texlab/main.tex",
                ),
                symbol: Symbol {
                    name: "3 Baz",
                    kind: Section,
                    label: Some(
                        Span(
                            "sec:baz",
                            308..323,
                        ),
                    ),
                    full_range: 295..445,
                    selection_range: 308..323,
                    children: [],
                },
            },
            SymbolLocation {
                document: Document(
                    "file:///texlab/main.tex",
                ),
                symbol: Symbol {
                    name: "4 Qux",
                    kind: Section,
                    label: Some(
                        Span(
                            "sec:qux",
                            460..475,
                        ),
                    ),
                    full_range: 447..557,
                    selection_range: 460..475,
                    children: [],
                },
            },
        ]
    "#]],
    );
}

#[test]
fn filter_type_figure() {
    check(
        "figure",
        expect![[r#"
        [
            SymbolLocation {
                document: Document(
                    "file:///texlab/main.tex",
                ),
                symbol: Symbol {
                    name: "Figure 1: Bar",
                    kind: Figure,
                    label: Some(
                        Span(
                            "fig:bar",
                            265..280,
                        ),
                    ),
                    full_range: 220..293,
                    selection_range: 265..280,
                    children: [],
                },
            },
        ]
    "#]],
    );
}

#[test]
fn filter_type_item() {
    check(
        "item",
        expect![[r#"
        [
            SymbolLocation {
                document: Document(
                    "file:///texlab/main.tex",
                ),
                symbol: Symbol {
                    name: "1",
                    kind: EnumerationItem,
                    label: Some(
                        Span(
                            "itm:foo",
                            352..367,
                        ),
                    ),
                    full_range: 347..371,
                    selection_range: 352..367,
                    children: [],
                },
            },
            SymbolLocation {
                document: Document(
                    "file:///texlab/main.tex",
                ),
                symbol: Symbol {
                    name: "2",
                    kind: EnumerationItem,
                    label: Some(
                        Span(
                            "itm:bar",
                            381..396,
                        ),
                    ),
                    full_range: 376..400,
                    selection_range: 381..396,
                    children: [],
                },
            },
            SymbolLocation {
                document: Document(
                    "file:///texlab/main.tex",
                ),
                symbol: Symbol {
                    name: "3",
                    kind: EnumerationItem,
                    label: Some(
                        Span(
                            "itm:baz",
                            410..425,
                        ),
                    ),
                    full_range: 405..429,
                    selection_range: 410..425,
                    children: [],
                },
            },
        ]
    "#]],
    );
}

#[test]
fn filter_type_math() {
    check(
        "math",
        expect![[r#"
        [
            SymbolLocation {
                document: Document(
                    "file:///texlab/main.tex",
                ),
                symbol: Symbol {
                    name: "Equation (1)",
                    kind: Equation,
                    label: Some(
                        Span(
                            "eq:foo",
                            151..165,
                        ),
                    ),
                    full_range: 135..188,
                    selection_range: 151..165,
                    children: [],
                },
            },
            SymbolLocation {
                document: Document(
                    "file:///texlab/main.tex",
                ),
                symbol: Symbol {
                    name: "Lemma 1 (Qux)",
                    kind: Theorem,
                    label: Some(
                        Span(
                            "thm:qux",
                            522..537,
                        ),
                    ),
                    full_range: 504..557,
                    selection_range: 522..537,
                    children: [],
                },
            },
        ]
    "#]],
    );
}

#[test]
fn filter_bibtex() {
    check(
        "bibtex",
        expect![[r#"
        [
            SymbolLocation {
                document: Document(
                    "file:///texlab/main.bib",
                ),
                symbol: Symbol {
                    name: "foo",
                    kind: Entry(
                        Article,
                    ),
                    label: None,
                    full_range: 0..14,
                    selection_range: 9..12,
                    children: [],
                },
            },
            SymbolLocation {
                document: Document(
                    "file:///texlab/main.bib",
                ),
                symbol: Symbol {
                    name: "bar",
                    kind: Entry(
                        String,
                    ),
                    label: None,
                    full_range: 16..36,
                    selection_range: 24..27,
                    children: [],
                },
            },
        ]
    "#]],
    );
}
