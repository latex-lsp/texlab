use base_db::{Config, SymbolConfig, SymbolEnvironmentConfig};
use expect_test::{Expect, expect};
use regex::Regex;
use rustc_hash::FxHashMap;
use test_utils::fixture::Fixture;

use crate::document_symbols;

fn check(fixture: &Fixture, expect: Expect) {
    let document = fixture.workspace.lookup(&fixture.documents[0].uri).unwrap();
    let symbols = document_symbols(&fixture.workspace, document);
    expect.assert_debug_eq(&symbols);
}

#[test]
fn test_enumerate() {
    let fixture = Fixture::parse(
        r#"
%! main.tex
\documentclass{article}

\begin{document}

\begin{enumerate}
    \item\label{it:foo} Foo
    \item\label{it:bar} Bar
    \item[Baz] Baz
    \item[Qux]\label{it:qux} Qux
\end{enumerate}

\end{document}

%! main.aux
\relax
\newlabel{it:foo}{{1}{1}}
\newlabel{it:qux}{{2}{1}}"#,
    );

    check(
        &fixture,
        expect![[r#"
        [
            Symbol {
                name: "Enumerate",
                kind: Enumeration,
                label: None,
                full_range: 43..184,
                selection_range: 43..184,
                children: [
                    Symbol {
                        name: "1",
                        kind: EnumerationItem,
                        label: Some(
                            Span(
                                "it:foo",
                                70..84,
                            ),
                        ),
                        full_range: 65..88,
                        selection_range: 70..84,
                        children: [],
                    },
                    Symbol {
                        name: "Item",
                        kind: EnumerationItem,
                        label: Some(
                            Span(
                                "it:bar",
                                98..112,
                            ),
                        ),
                        full_range: 93..116,
                        selection_range: 98..112,
                        children: [],
                    },
                    Symbol {
                        name: "Baz",
                        kind: EnumerationItem,
                        label: None,
                        full_range: 121..135,
                        selection_range: 121..135,
                        children: [],
                    },
                    Symbol {
                        name: "2",
                        kind: EnumerationItem,
                        label: Some(
                            Span(
                                "it:qux",
                                150..164,
                            ),
                        ),
                        full_range: 140..168,
                        selection_range: 150..164,
                        children: [],
                    },
                ],
            },
        ]
    "#]],
    );
}

#[test]
fn test_equation() {
    let fixture = Fixture::parse(
        r#"
%! main.tex
\documentclass{article}

\begin{document}

\begin{equation}\label{eq:foo}
    Foo
\end{equation}

\begin{equation}\label{eq:bar}
    Bar
\end{equation}

\begin{equation}
    Baz
\end{equation}

\end{document}

%! main.aux
\relax
\newlabel{eq:foo}{{1}{1}}"#,
    );

    check(
        &fixture,
        expect![[r#"
        [
            Symbol {
                name: "Equation (1)",
                kind: Equation,
                label: Some(
                    Span(
                        "eq:foo",
                        59..73,
                    ),
                ),
                full_range: 43..96,
                selection_range: 59..73,
                children: [],
            },
            Symbol {
                name: "Equation",
                kind: Equation,
                label: Some(
                    Span(
                        "eq:bar",
                        114..128,
                    ),
                ),
                full_range: 98..151,
                selection_range: 114..128,
                children: [],
            },
            Symbol {
                name: "Equation",
                kind: Equation,
                label: None,
                full_range: 153..192,
                selection_range: 153..192,
                children: [],
            },
        ]
    "#]],
    );
}

#[test]
fn test_float() {
    let fixture = Fixture::parse(
        r#"
%! main.tex
\documentclass{article}

\begin{document}

\begin{figure}
    Foo
    \caption{Foo}\label{fig:foo}
\end{figure}

\begin{figure}
    Bar
    \caption{Bar}\label{fig:bar}
\end{figure}

\begin{figure}
    Baz
    \caption{Baz}
\end{figure}

\begin{figure*}
    Baz
    \caption{Baz2}
\end{figure*}

\begin{figure}
    Qux
\end{figure}

\end{document}
|

%! main.aux
\relax
\@writefile{lof}{\contentsline {figure}{\numberline {1}{\ignorespaces Foo}}{1}\protected@file@percent }
\newlabel{fig:foo}{{1}{1}}
\@writefile{lof}{\contentsline {figure}{\numberline {2}{\ignorespaces Bar}}{1}\protected@file@percent }
\@writefile{lof}{\contentsline {figure}{\numberline {3}{\ignorespaces Baz}}{1}\protected@file@percent }"#,
    );

    check(
        &fixture,
        expect![[r#"
        [
            Symbol {
                name: "Figure 1: Foo",
                kind: Figure,
                label: Some(
                    Span(
                        "fig:foo",
                        83..98,
                    ),
                ),
                full_range: 43..111,
                selection_range: 83..98,
                children: [],
            },
            Symbol {
                name: "Figure: Bar",
                kind: Figure,
                label: Some(
                    Span(
                        "fig:bar",
                        153..168,
                    ),
                ),
                full_range: 113..181,
                selection_range: 153..168,
                children: [],
            },
            Symbol {
                name: "Figure: Baz",
                kind: Figure,
                label: None,
                full_range: 183..236,
                selection_range: 183..236,
                children: [],
            },
            Symbol {
                name: "Figure: Baz2",
                kind: Figure,
                label: None,
                full_range: 238..294,
                selection_range: 238..294,
                children: [],
            },
        ]
    "#]],
    );
}

#[test]
fn test_section() {
    let fixture = Fixture::parse(
        r#"
%! main.tex
\documentclass{article}

\begin{document}

\section{Foo}

\section{Bar}\label{sec:bar}

\subsection{Baz}\label{sec:baz}

\end{document}
|

%! main.aux
\relax
\@writefile{toc}{\contentsline {section}{\numberline {1}Foo}{1}\protected@file@percent }
\@writefile{toc}{\contentsline {section}{\numberline {2}Bar}{1}\protected@file@percent }
\newlabel{sec:bar}{{2}{1}}"#,
    );

    check(
        &fixture,
        expect![[r#"
            [
                Symbol {
                    name: "1 Foo",
                    kind: Section,
                    label: None,
                    full_range: 43..56,
                    selection_range: 43..56,
                    children: [],
                },
                Symbol {
                    name: "2 Bar",
                    kind: Section,
                    label: Some(
                        Span(
                            "sec:bar",
                            71..86,
                        ),
                    ),
                    full_range: 58..119,
                    selection_range: 71..86,
                    children: [
                        Symbol {
                            name: "Baz",
                            kind: Section,
                            label: Some(
                                Span(
                                    "sec:baz",
                                    104..119,
                                ),
                            ),
                            full_range: 88..119,
                            selection_range: 104..119,
                            children: [],
                        },
                    ],
                },
            ]
        "#]],
    );
}

#[test]
fn test_theorem_amsthm() {
    let fixture = Fixture::parse(
        r#"
%! main.tex
\documentclass{article}
\usepackage{amsthm}
\newtheorem{lemma}{Lemma}

\begin{document}

\begin{lemma}[Foo]\label{thm:foo}
    Foo
\end{lemma}

\begin{lemma}\label{thm:bar}
    Bar
\end{lemma}

\begin{lemma}\label{thm:baz}
    Baz
\end{lemma}

\begin{lemma}[Qux]
    Qux
\end{lemma}

\end{document}
|

%! main.aux
\relax
\newlabel{thm:foo}{{1}{1}}
\newlabel{thm:bar}{{2}{1}}"#,
    );

    check(
        &fixture,
        expect![[r#"
        [
            Symbol {
                name: "Lemma 1 (Foo)",
                kind: Theorem,
                label: Some(
                    Span(
                        "thm:foo",
                        107..122,
                    ),
                ),
                full_range: 89..142,
                selection_range: 107..122,
                children: [],
            },
            Symbol {
                name: "Lemma 2",
                kind: Theorem,
                label: Some(
                    Span(
                        "thm:bar",
                        157..172,
                    ),
                ),
                full_range: 144..192,
                selection_range: 157..172,
                children: [],
            },
            Symbol {
                name: "Lemma",
                kind: Theorem,
                label: Some(
                    Span(
                        "thm:baz",
                        207..222,
                    ),
                ),
                full_range: 194..242,
                selection_range: 207..222,
                children: [],
            },
            Symbol {
                name: "Lemma (Qux)",
                kind: Theorem,
                label: None,
                full_range: 244..282,
                selection_range: 244..282,
                children: [],
            },
        ]
    "#]],
    );
}

#[test]
fn test_theorem_thmtools() {
    let fixture = Fixture::parse(
        r#"
%! main.tex
\documentclass{article}
\declaretheoremstyle{lemmastyle}
\declaretheorem[style=lemmastyle, name=Lemma]{lemma}

\begin{document}

\begin{lemma}[Foo]\label{thm:foo}
    Foo
\end{lemma}

\begin{lemma}\label{thm:bar}
    Bar
\end{lemma}

\begin{lemma}\label{thm:baz}
    Baz
\end{lemma}

\begin{lemma}[Qux]
    Qux
\end{lemma}

\end{document}
|

%! main.aux
\relax
\newlabel{thm:foo}{{1}{1}}
\newlabel{thm:bar}{{2}{1}}"#,
    );

    check(
        &fixture,
        expect![[r#"
        [
            Symbol {
                name: "Lemma 1 (Foo)",
                kind: Theorem,
                label: Some(
                    Span(
                        "thm:foo",
                        147..162,
                    ),
                ),
                full_range: 129..182,
                selection_range: 147..162,
                children: [],
            },
            Symbol {
                name: "Lemma 2",
                kind: Theorem,
                label: Some(
                    Span(
                        "thm:bar",
                        197..212,
                    ),
                ),
                full_range: 184..232,
                selection_range: 197..212,
                children: [],
            },
            Symbol {
                name: "Lemma",
                kind: Theorem,
                label: Some(
                    Span(
                        "thm:baz",
                        247..262,
                    ),
                ),
                full_range: 234..282,
                selection_range: 247..262,
                children: [],
            },
            Symbol {
                name: "Lemma (Qux)",
                kind: Theorem,
                label: None,
                full_range: 284..322,
                selection_range: 284..322,
                children: [],
            },
        ]
    "#]],
    );
}

#[test]
fn test_allowed_patterns() {
    let mut fixture = Fixture::parse(
        r#"
%! main.tex
\documentclass{article}

\begin{document}

\begin{equation}\label{eq:foo}
    Foo
\end{equation}

\begin{enumerate}
    \item Foo
    \item Bar
\end{enumerate}

\end{document}"#,
    );

    fixture.workspace.set_config(Config {
        symbols: SymbolConfig {
            allowed_patterns: vec![
                Regex::new("Item").unwrap(),
                Regex::new("Enumerate").unwrap(),
            ],
            ignored_patterns: Vec::new(),
            custom_environments: FxHashMap::default(),
        },
        ..Config::default()
    });

    check(
        &fixture,
        expect![[r#"
        [
            Symbol {
                name: "Enumerate",
                kind: Enumeration,
                label: None,
                full_range: 98..159,
                selection_range: 98..159,
                children: [
                    Symbol {
                        name: "Item",
                        kind: EnumerationItem,
                        label: None,
                        full_range: 120..129,
                        selection_range: 120..129,
                        children: [],
                    },
                    Symbol {
                        name: "Item",
                        kind: EnumerationItem,
                        label: None,
                        full_range: 134..143,
                        selection_range: 134..143,
                        children: [],
                    },
                ],
            },
        ]
    "#]],
    );
}

#[test]
fn test_ignored_patterns() {
    let mut fixture = Fixture::parse(
        r#"
%! main.tex
\documentclass{article}

\begin{document}

\begin{equation}\label{eq:foo}
    Foo
\end{equation}

\begin{enumerate}
    \item Foo
    \item Bar
\end{enumerate}

\end{document}"#,
    );

    fixture.workspace.set_config(Config {
        symbols: SymbolConfig {
            ignored_patterns: vec![
                Regex::new("Item").unwrap(),
                Regex::new("Enumerate").unwrap(),
            ],
            allowed_patterns: Vec::new(),
            custom_environments: FxHashMap::default(),
        },
        ..Config::default()
    });

    check(
        &fixture,
        expect![[r#"
        [
            Symbol {
                name: "Equation",
                kind: Equation,
                label: Some(
                    Span(
                        "eq:foo",
                        59..73,
                    ),
                ),
                full_range: 43..96,
                selection_range: 59..73,
                children: [],
            },
        ]
    "#]],
    );
}

#[test]
fn test_custom_environments() {
    let mut fixture = Fixture::parse(
        r#"
%! main.tex
\documentclass{article}

\begin{document}

\begin{foo}
\begin{equation}\label{eq:foo}
    Foo
\end{equation}
\end{foo}

\end{document}"#,
    );

    fixture.workspace.set_config(Config {
        symbols: SymbolConfig {
            ignored_patterns: vec![
                Regex::new("Item").unwrap(),
                Regex::new("Enumerate").unwrap(),
            ],
            allowed_patterns: Vec::new(),
            custom_environments: FxHashMap::from_iter([(
                "foo".into(),
                SymbolEnvironmentConfig {
                    display_name: "Foo".into(),
                    label: false,
                },
            )]),
        },
        ..Config::default()
    });

    check(
        &fixture,
        expect![[r#"
        [
            Symbol {
                name: "Foo",
                kind: Environment,
                label: None,
                full_range: 43..118,
                selection_range: 43..118,
                children: [
                    Symbol {
                        name: "Equation",
                        kind: Equation,
                        label: Some(
                            Span(
                                "eq:foo",
                                71..85,
                            ),
                        ),
                        full_range: 55..108,
                        selection_range: 71..85,
                        children: [],
                    },
                ],
            },
        ]
    "#]],
    );
}

#[test]
fn test_command_definition() {
    let fixture = Fixture::parse(
        r#"
%! main.sty
\newcommand{\foo}{foo}

\let\bar\baz"#,
    );

    check(
        &fixture,
        expect![[r#"
            [
                Symbol {
                    name: "define \\foo",
                    kind: CommandDefinition,
                    label: None,
                    full_range: 0..22,
                    selection_range: 12..16,
                    children: [],
                },
                Symbol {
                    name: "define \\bar",
                    kind: CommandDefinition,
                    label: None,
                    full_range: 24..32,
                    selection_range: 28..32,
                    children: [],
                },
            ]
        "#]],
    );
}
