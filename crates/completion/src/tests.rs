use base_db::FeatureParams;
use expect_test::{expect, Expect};

use crate::CompletionParams;

fn check(input: &str, expect: Expect) {
    let fixture = test_utils::fixture::Fixture::parse(input);

    let (offset, spec) = fixture
        .documents
        .iter()
        .find_map(|document| Some((document.cursor?, document)))
        .unwrap();

    let document = fixture.workspace.lookup(&spec.uri).unwrap();
    let feature = FeatureParams::new(&fixture.workspace, document);
    let params = CompletionParams { feature, offset };
    let mut result = crate::complete(&params);
    result.items.truncate(5);
    expect.assert_debug_eq(&result.items);
}

#[test]
fn acronym_ref_simple() {
    check(
        r#"
%! main.tex
\newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
\acrshort{f}
          |
          ^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 31,
                    range: 87..88,
                    preselect: false,
                    data: GlossaryEntry(
                        GlossaryEntryData {
                            name: "fpsLabel",
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn acronym_ref_empty() {
    check(
        r#"
%! main.tex
\newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
\acrshort{}
          |"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 0,
                    range: 87..87,
                    preselect: false,
                    data: GlossaryEntry(
                        GlossaryEntryData {
                            name: "fpsLabel",
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn acronym_ref_after_group() {
    check(
        r#"
%! main.tex
\newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
\acrshort{}
           |"#,
        expect![[r#"
            []
        "#]],
    );
}

#[test]
fn acronym_ref_open_brace() {
    check(
        r#"
%! main.tex
\newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
\acrshort{f
          |
          ^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 31,
                    range: 87..88,
                    preselect: false,
                    data: GlossaryEntry(
                        GlossaryEntryData {
                            name: "fpsLabel",
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn acronym_package_ref() {
    check(
        r#"
%! main.tex
\acrodef{fpsLabel}[FPS]{Frames per Second}
\ac{f
    |
    ^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 31,
                    range: 47..48,
                    preselect: false,
                    data: GlossaryEntry(
                        GlossaryEntryData {
                            name: "fpsLabel",
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn glossary_ref_simple() {
    check(
        r#"
%! main.tex
\newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
\gls{f}
     |
     ^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 31,
                    range: 82..83,
                    preselect: false,
                    data: GlossaryEntry(
                        GlossaryEntryData {
                            name: "fpsLabel",
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn glossary_ref_open_brace() {
    check(
        r#"
%! main.tex
\newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
\gls{f
     |
     ^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 31,
                    range: 82..83,
                    preselect: false,
                    data: GlossaryEntry(
                        GlossaryEntryData {
                            name: "fpsLabel",
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn argument_empty() {
    check(
        r#"
%! main.tex
\usepackage{amsfonts}
\mathbb{}
        |"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 0,
                    range: 30..30,
                    preselect: false,
                    data: Argument(
                        ArgumentData(
                            "A",
                        ),
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 30..30,
                    preselect: false,
                    data: Argument(
                        ArgumentData(
                            "B",
                        ),
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 30..30,
                    preselect: false,
                    data: Argument(
                        ArgumentData(
                            "C",
                        ),
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 30..30,
                    preselect: false,
                    data: Argument(
                        ArgumentData(
                            "D",
                        ),
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 30..30,
                    preselect: false,
                    data: Argument(
                        ArgumentData(
                            "E",
                        ),
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn argument_word() {
    check(
        r#"
%! main.tex
\usepackage{amsfonts}
\mathbb{A}
        |
        ^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 31,
                    range: 30..31,
                    preselect: false,
                    data: Argument(
                        ArgumentData(
                            "A",
                        ),
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn argument_open_brace() {
    check(
        r#"
%! main.tex
\usepackage{amsfonts}
\mathbb{
        |
Test"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 0,
                    range: 30..30,
                    preselect: false,
                    data: Argument(
                        ArgumentData(
                            "A",
                        ),
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 30..30,
                    preselect: false,
                    data: Argument(
                        ArgumentData(
                            "B",
                        ),
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 30..30,
                    preselect: false,
                    data: Argument(
                        ArgumentData(
                            "C",
                        ),
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 30..30,
                    preselect: false,
                    data: Argument(
                        ArgumentData(
                            "D",
                        ),
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 30..30,
                    preselect: false,
                    data: Argument(
                        ArgumentData(
                            "E",
                        ),
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn argument_open_brace_unrelated() {
    check(
        r#"
%! main.tex
\usepackage{amsfonts}
\mathbb{}{
          |
Test"#,
        expect![[r#"
            []
        "#]],
    );
}

#[test]
fn begin_environment_without_snippet_support() {
    check(
        r#"
%! main.tex
\beg
    |
 ^^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 71,
                    range: 1..4,
                    preselect: false,
                    data: BeginEnvironment,
                },
                CompletionItem {
                    score: 71,
                    range: 1..4,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "begingroup",
                            package: [],
                        },
                    ),
                },
                CompletionItem {
                    score: 59,
                    range: 1..4,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "AtBeginDocument",
                            package: [],
                        },
                    ),
                },
                CompletionItem {
                    score: 59,
                    range: 1..4,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "AtBeginDvi",
                            package: [],
                        },
                    ),
                },
                CompletionItem {
                    score: 53,
                    range: 1..4,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "bigwedge",
                            package: [],
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn citation() {
    check(
        r#"
%! main.tex
\documentclass{article}
\bibliography{main}
\begin{document}
\cite{
      |
\end{document}

%! main.bib
@article{foo:2019,
    author = {Foo Bar},
    title = {Baz Qux},
    year = {2019},
}

@article{bar:2005,}"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 0,
                    range: 67..67,
                    preselect: false,
                    data: Citation(
                        CitationData {
                            document: Document(
                                "file:///texlab/main.bib",
                            ),
                            entry: Entry {
                                name: Span(
                                    "bar:2005",
                                    97..105,
                                ),
                                full_range: 88..107,
                                keywords: "bar:2005 @article",
                                category: Article,
                            },
                        },
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 67..67,
                    preselect: false,
                    data: Citation(
                        CitationData {
                            document: Document(
                                "file:///texlab/main.bib",
                            ),
                            entry: Entry {
                                name: Span(
                                    "foo:2019",
                                    9..17,
                                ),
                                full_range: 0..86,
                                keywords: "foo:2019 @article Foo Bar Baz Qux 2019",
                                category: Article,
                            },
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn citation_open_brace() {
    check(
        r#"
%! main.tex
\addbibresource{main.bib}
\cite{
      |

%! main.bib
@article{foo,}"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 0,
                    range: 32..32,
                    preselect: false,
                    data: Citation(
                        CitationData {
                            document: Document(
                                "file:///texlab/main.bib",
                            ),
                            entry: Entry {
                                name: Span(
                                    "foo",
                                    9..12,
                                ),
                                full_range: 0..14,
                                keywords: "foo @article",
                                category: Article,
                            },
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn citation_open_brace_multiple() {
    check(
        r#"
%! main.tex
\addbibresource{main.bib}
\cite{foo,f
          |
          ^

%! main.bib
@article{foo,}"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 31,
                    range: 36..37,
                    preselect: false,
                    data: Citation(
                        CitationData {
                            document: Document(
                                "file:///texlab/main.bib",
                            ),
                            entry: Entry {
                                name: Span(
                                    "foo",
                                    9..12,
                                ),
                                full_range: 0..14,
                                keywords: "foo @article",
                                category: Article,
                            },
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn citation_acronym() {
    check(
        r#"
%! main.tex
\addbibresource{main.bib}
\DeclareAcronym{foo}{cite={}}
                           |

%! main.bib
@article{foo,}"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 0,
                    range: 53..53,
                    preselect: false,
                    data: Citation(
                        CitationData {
                            document: Document(
                                "file:///texlab/main.bib",
                            ),
                            entry: Entry {
                                name: Span(
                                    "foo",
                                    9..12,
                                ),
                                full_range: 0..14,
                                keywords: "foo @article",
                                category: Article,
                            },
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn citation_after_brace() {
    check(
        r#"
%! main.tex
\documentclass{article}
\bibliography{main}
\begin{document}
\cite{}
       |
\end{document}

%! main.bib
@article{foo,}"#,
        expect![[r#"
            []
        "#]],
    );
}

#[test]
fn color_model_definition_simple() {
    check(
        r#"
%! main.tex
\definecolor{foo}{}
                  |"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 0,
                    range: 18..18,
                    preselect: false,
                    data: ColorModel(
                        "HTML",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 18..18,
                    preselect: false,
                    data: ColorModel(
                        "RGB",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 18..18,
                    preselect: false,
                    data: ColorModel(
                        "cmyk",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 18..18,
                    preselect: false,
                    data: ColorModel(
                        "gray",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 18..18,
                    preselect: false,
                    data: ColorModel(
                        "rgb",
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn color_model_definition_open_brace() {
    check(
        r#"
%! main.tex
\definecolor{foo}{
                  |"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 0,
                    range: 18..18,
                    preselect: false,
                    data: ColorModel(
                        "HTML",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 18..18,
                    preselect: false,
                    data: ColorModel(
                        "RGB",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 18..18,
                    preselect: false,
                    data: ColorModel(
                        "cmyk",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 18..18,
                    preselect: false,
                    data: ColorModel(
                        "gray",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 18..18,
                    preselect: false,
                    data: ColorModel(
                        "rgb",
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn color_model_definition_set_simple() {
    check(
        r#"
%! main.tex
\definecolorset{}
                |"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 0,
                    range: 16..16,
                    preselect: false,
                    data: ColorModel(
                        "HTML",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 16..16,
                    preselect: false,
                    data: ColorModel(
                        "RGB",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 16..16,
                    preselect: false,
                    data: ColorModel(
                        "cmyk",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 16..16,
                    preselect: false,
                    data: ColorModel(
                        "gray",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 16..16,
                    preselect: false,
                    data: ColorModel(
                        "rgb",
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn color_model_definition_set_open_brace() {
    check(
        r#"
%! main.tex
\definecolorset{
                |"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 0,
                    range: 16..16,
                    preselect: false,
                    data: ColorModel(
                        "HTML",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 16..16,
                    preselect: false,
                    data: ColorModel(
                        "RGB",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 16..16,
                    preselect: false,
                    data: ColorModel(
                        "cmyk",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 16..16,
                    preselect: false,
                    data: ColorModel(
                        "gray",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 16..16,
                    preselect: false,
                    data: ColorModel(
                        "rgb",
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn color_simple() {
    check(
        r#"
%! main.tex
\color{}
       |"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 0,
                    range: 7..7,
                    preselect: false,
                    data: Color(
                        "Apricot",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 7..7,
                    preselect: false,
                    data: Color(
                        "Aquamarine",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 7..7,
                    preselect: false,
                    data: Color(
                        "Bittersweet",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 7..7,
                    preselect: false,
                    data: Color(
                        "Black",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 7..7,
                    preselect: false,
                    data: Color(
                        "Blue",
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn color_word() {
    check(
        r#"
%! main.tex
\color{re}
        |
       ^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 51,
                    range: 7..9,
                    preselect: false,
                    data: Color(
                        "red",
                    ),
                },
                CompletionItem {
                    score: 49,
                    range: 7..9,
                    preselect: false,
                    data: Color(
                        "Red",
                    ),
                },
                CompletionItem {
                    score: 49,
                    range: 7..9,
                    preselect: false,
                    data: Color(
                        "RedOrange",
                    ),
                },
                CompletionItem {
                    score: 49,
                    range: 7..9,
                    preselect: false,
                    data: Color(
                        "RedViolet",
                    ),
                },
                CompletionItem {
                    score: 39,
                    range: 7..9,
                    preselect: false,
                    data: Color(
                        "BrickRed",
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn color_open_brace() {
    check(
        r#"
%! main.tex
\color{
       |"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 0,
                    range: 7..7,
                    preselect: false,
                    data: Color(
                        "Apricot",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 7..7,
                    preselect: false,
                    data: Color(
                        "Aquamarine",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 7..7,
                    preselect: false,
                    data: Color(
                        "Bittersweet",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 7..7,
                    preselect: false,
                    data: Color(
                        "Black",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 7..7,
                    preselect: false,
                    data: Color(
                        "Blue",
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn component_command_simple() {
    check(
        r#"
%! main.tex
\
 |"#,
        expect![[r##"
            [
                CompletionItem {
                    score: 0,
                    range: 1..1,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "!",
                            package: [],
                        },
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 1..1,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "\"",
                            package: [],
                        },
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 1..1,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "#",
                            package: [],
                        },
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 1..1,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "$",
                            package: [],
                        },
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 1..1,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "%",
                            package: [],
                        },
                    ),
                },
            ]
        "##]],
    );
}

#[test]
fn component_command_simple_before() {
    check(
        r#"
%! main.tex
\
|"#,
        expect![[r#"
            []
        "#]],
    );
}

#[test]
fn component_command_simple_package() {
    check(
        r#"
%! main.tex
\usepackage{lipsum}
\lips
   |
 ^^^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 91,
                    range: 21..25,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "lipsum",
                            package: [
                                "lipsum.sty",
                            ],
                        },
                    ),
                },
                CompletionItem {
                    score: 91,
                    range: 21..25,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "lipsumexp",
                            package: [
                                "lipsum.sty",
                            ],
                        },
                    ),
                },
                CompletionItem {
                    score: 89,
                    range: 21..25,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "LipsumPar",
                            package: [
                                "lipsum.sty",
                            ],
                        },
                    ),
                },
                CompletionItem {
                    score: 89,
                    range: 21..25,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "LipsumProtect",
                            package: [
                                "lipsum.sty",
                            ],
                        },
                    ),
                },
                CompletionItem {
                    score: 89,
                    range: 21..25,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "LipsumRestoreAll",
                            package: [
                                "lipsum.sty",
                            ],
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn component_command_bibtex() {
    check(
        r#"
%! main.bib
@article{b,
    c = {\LaT }
           |
          ^^^
}"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 73,
                    range: 22..25,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "LaTeX",
                            package: [],
                        },
                    ),
                },
                CompletionItem {
                    score: 73,
                    range: 22..25,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "LaTeXe",
                            package: [],
                        },
                    ),
                },
                CompletionItem {
                    score: 67,
                    range: 22..25,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "latexreleaseversion",
                            package: [],
                        },
                    ),
                },
                CompletionItem {
                    score: 61,
                    range: 22..25,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "LastDeclaredEncoding",
                            package: [],
                        },
                    ),
                },
                CompletionItem {
                    score: 59,
                    range: 22..25,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "last",
                            package: [],
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn component_environment_simple() {
    check(
        r#"
%! main.tex
\begin{doc
          |
       ^^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 71,
                    range: 7..10,
                    preselect: false,
                    data: Environment(
                        EnvironmentData {
                            name: "document",
                            package: [],
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn component_environment_simple_end() {
    check(
        r#"
%! main.tex
\begin{document}
\end{
     |"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 0,
                    range: 22..22,
                    preselect: true,
                    data: Environment(
                        EnvironmentData {
                            name: "document",
                            package: [],
                        },
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 22..22,
                    preselect: false,
                    data: Environment(
                        EnvironmentData {
                            name: "abstract",
                            package: [],
                        },
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 22..22,
                    preselect: false,
                    data: Environment(
                        EnvironmentData {
                            name: "array",
                            package: [],
                        },
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 22..22,
                    preselect: false,
                    data: Environment(
                        EnvironmentData {
                            name: "center",
                            package: [],
                        },
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 22..22,
                    preselect: false,
                    data: Environment(
                        EnvironmentData {
                            name: "csname",
                            package: [],
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn component_environment_class() {
    check(
        r#"
%! main.tex
\documentclass{article}
\begin{thein}
          |
       ^^^^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 111,
                    range: 31..36,
                    preselect: false,
                    data: Environment(
                        EnvironmentData {
                            name: "theindex",
                            package: [
                                "article.cls",
                            ],
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn component_environment_command_definition() {
    check(
        r#"
%! main.tex
\newcommand{\foo}{\begin{doc}
                           |
                         ^^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 71,
                    range: 25..28,
                    preselect: false,
                    data: Environment(
                        EnvironmentData {
                            name: "document",
                            package: [],
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn entry_type_at_empty() {
    check(
        r#"
%! main.bib
@
 |"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 0,
                    range: 1..1,
                    preselect: false,
                    data: EntryType(
                        EntryTypeData(
                            "article",
                        ),
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 1..1,
                    preselect: false,
                    data: EntryType(
                        EntryTypeData(
                            "artwork",
                        ),
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 1..1,
                    preselect: false,
                    data: EntryType(
                        EntryTypeData(
                            "audio",
                        ),
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 1..1,
                    preselect: false,
                    data: EntryType(
                        EntryTypeData(
                            "bibnote",
                        ),
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 1..1,
                    preselect: false,
                    data: EntryType(
                        EntryTypeData(
                            "book",
                        ),
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn entry_type_before_preamble() {
    check(
        r#"
%! main.bib
@preamble
 |
 ^^^^^^^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 171,
                    range: 1..9,
                    preselect: false,
                    data: EntryType(
                        EntryTypeData(
                            "preamble",
                        ),
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn entry_type_before_string() {
    check(
        r#"
%! main.bib
@string
 |
 ^^^^^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 131,
                    range: 1..7,
                    preselect: false,
                    data: EntryType(
                        EntryTypeData(
                            "string",
                        ),
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn entry_type_before_article() {
    check(
        r#"
%! main.bib
@article
 |
 ^^^^^^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 151,
                    range: 1..8,
                    preselect: false,
                    data: EntryType(
                        EntryTypeData(
                            "article",
                        ),
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn entry_type_after_preamble() {
    check(
        r#"
%! main.bib
@preamble{
         |
 ^^^^^^^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 171,
                    range: 1..9,
                    preselect: false,
                    data: EntryType(
                        EntryTypeData(
                            "preamble",
                        ),
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn entry_type_after_string() {
    check(
        r#"
%! main.bib
@string{
       |
 ^^^^^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 131,
                    range: 1..7,
                    preselect: false,
                    data: EntryType(
                        EntryTypeData(
                            "string",
                        ),
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn entry_type_complete_entry() {
    check(
        r#"
%! main.bib
@article{foo, author = {foo}}
   |
 ^^^^^^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 151,
                    range: 1..8,
                    preselect: false,
                    data: EntryType(
                        EntryTypeData(
                            "article",
                        ),
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn field_empty_entry_open() {
    check(
        r#"
%! main.bib
@article{foo,
             |"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 0,
                    range: 13..13,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "abstract",
                        ),
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 13..13,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "addendum",
                        ),
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 13..13,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "address",
                        ),
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 13..13,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "afterword",
                        ),
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 13..13,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "annotation",
                        ),
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn field_empty_entry_closed() {
    check(
        r#"
%! main.bib
@article{foo,}
             |"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 0,
                    range: 13..13,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "abstract",
                        ),
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 13..13,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "addendum",
                        ),
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 13..13,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "address",
                        ),
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 13..13,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "afterword",
                        ),
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 13..13,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "annotation",
                        ),
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn field_entry_field_name() {
    check(
        r#"
%! main.bib
@article{foo, a
               |
              ^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 31,
                    range: 14..15,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "abstract",
                        ),
                    ),
                },
                CompletionItem {
                    score: 31,
                    range: 14..15,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "addendum",
                        ),
                    ),
                },
                CompletionItem {
                    score: 31,
                    range: 14..15,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "address",
                        ),
                    ),
                },
                CompletionItem {
                    score: 31,
                    range: 14..15,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "afterword",
                        ),
                    ),
                },
                CompletionItem {
                    score: 31,
                    range: 14..15,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "annotation",
                        ),
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn field_entry_two_fields_name_open() {
    check(
        r#"
%! main.bib
@article{foo, author = bar, edit
                             |
                            ^^^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 91,
                    range: 28..32,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "edition",
                        ),
                    ),
                },
                CompletionItem {
                    score: 91,
                    range: 28..32,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "editor",
                        ),
                    ),
                },
                CompletionItem {
                    score: 91,
                    range: 28..32,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "editora",
                        ),
                    ),
                },
                CompletionItem {
                    score: 91,
                    range: 28..32,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "editoratype",
                        ),
                    ),
                },
                CompletionItem {
                    score: 91,
                    range: 28..32,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "editorb",
                        ),
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn field_entry_two_fields_name_closed() {
    check(
        r#"
%! main.bib
@article{foo, author = bar, edit}
                             |
                            ^^^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 91,
                    range: 28..32,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "edition",
                        ),
                    ),
                },
                CompletionItem {
                    score: 91,
                    range: 28..32,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "editor",
                        ),
                    ),
                },
                CompletionItem {
                    score: 91,
                    range: 28..32,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "editora",
                        ),
                    ),
                },
                CompletionItem {
                    score: 91,
                    range: 28..32,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "editoratype",
                        ),
                    ),
                },
                CompletionItem {
                    score: 91,
                    range: 28..32,
                    preselect: false,
                    data: Field(
                        FieldTypeData(
                            "editorb",
                        ),
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn import_package_open_brace() {
    check(
        r#"
%! main.tex
\usepackage{lips
             |
            ^^^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 91,
                    range: 12..16,
                    preselect: false,
                    data: Package(
                        "lips",
                    ),
                },
                CompletionItem {
                    score: 91,
                    range: 12..16,
                    preselect: false,
                    data: Package(
                        "lipsum",
                    ),
                },
                CompletionItem {
                    score: 82,
                    range: 12..16,
                    preselect: false,
                    data: Package(
                        "lisp-simple-alloc",
                    ),
                },
                CompletionItem {
                    score: 82,
                    range: 12..16,
                    preselect: false,
                    data: Package(
                        "lisp-string",
                    ),
                },
                CompletionItem {
                    score: 82,
                    range: 12..16,
                    preselect: false,
                    data: Package(
                        "lwarp-lips",
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn import_package_closed_brace() {
    check(
        r#"
%! main.tex
\usepackage{lips}
             |
            ^^^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 91,
                    range: 12..16,
                    preselect: false,
                    data: Package(
                        "lips",
                    ),
                },
                CompletionItem {
                    score: 91,
                    range: 12..16,
                    preselect: false,
                    data: Package(
                        "lipsum",
                    ),
                },
                CompletionItem {
                    score: 82,
                    range: 12..16,
                    preselect: false,
                    data: Package(
                        "lisp-simple-alloc",
                    ),
                },
                CompletionItem {
                    score: 82,
                    range: 12..16,
                    preselect: false,
                    data: Package(
                        "lisp-string",
                    ),
                },
                CompletionItem {
                    score: 82,
                    range: 12..16,
                    preselect: false,
                    data: Package(
                        "lwarp-lips",
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn import_class_open_brace() {
    check(
        r#"
%! main.tex
\documentclass{art \foo
                |
               ^^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 71,
                    range: 15..18,
                    preselect: false,
                    data: DocumentClass(
                        "article",
                    ),
                },
                CompletionItem {
                    score: 71,
                    range: 15..18,
                    preselect: false,
                    data: DocumentClass(
                        "articleingud",
                    ),
                },
                CompletionItem {
                    score: 71,
                    range: 15..18,
                    preselect: false,
                    data: DocumentClass(
                        "articoletteracdp",
                    ),
                },
                CompletionItem {
                    score: 71,
                    range: 15..18,
                    preselect: false,
                    data: DocumentClass(
                        "artikel1",
                    ),
                },
                CompletionItem {
                    score: 71,
                    range: 15..18,
                    preselect: false,
                    data: DocumentClass(
                        "artikel2",
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn import_class_closed_brace() {
    check(
        r#"
%! main.tex
\documentclass{art}
                |
               ^^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 71,
                    range: 15..18,
                    preselect: false,
                    data: DocumentClass(
                        "article",
                    ),
                },
                CompletionItem {
                    score: 71,
                    range: 15..18,
                    preselect: false,
                    data: DocumentClass(
                        "articleingud",
                    ),
                },
                CompletionItem {
                    score: 71,
                    range: 15..18,
                    preselect: false,
                    data: DocumentClass(
                        "articoletteracdp",
                    ),
                },
                CompletionItem {
                    score: 71,
                    range: 15..18,
                    preselect: false,
                    data: DocumentClass(
                        "artikel1",
                    ),
                },
                CompletionItem {
                    score: 71,
                    range: 15..18,
                    preselect: false,
                    data: DocumentClass(
                        "artikel2",
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn label() {
    check(
        r#"
%! foo.tex
\documentclass{article}

\usepackage{amsmath}
\usepackage{caption}
\usepackage{amsthm}
\newtheorem{lemma}{Lemma}

\begin{document}

\section{Foo}%
\label{sec:foo}

\begin{equation}%
\label{eq:foo}
    1 + 1 = 2
\end{equation}

\begin{equation}%
\label{eq:bar}
    1 + 1 = 2
\end{equation}

\begin{figure}%
\LaTeX{}
\caption{Baz}%
\label{fig:baz}
\end{figure}

\begin{lemma}%
\label{thm:foo}
    1 + 1 = 2
\end{lemma}

\include{bar}

\end{document}

%! bar.tex
\section{Bar}%
\label{sec:bar}

Lorem ipsum dolor sit amet.
\ref{}
     |

%! foo.aux
\relax
\@writefile{lof}{\contentsline {figure}{\numberline {1}{\ignorespaces Baz\relax }}{1}\protected@file@percent }
\providecommand*\caption@xref[2]{\@setref\relax\@undefined{#1}}
\newlabel{fig:baz}{{1}{1}}
\@writefile{toc}{\contentsline {section}{\numberline {1}Foo}{1}\protected@file@percent }
\newlabel{sec:foo}{{1}{1}}
\newlabel{eq:foo}{{1}{1}}
\newlabel{eq:bar}{{2}{1}}
\newlabel{thm:foo}{{1}{1}}
\@input{bar.aux}"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 0,
                    range: 65..65,
                    preselect: false,
                    data: Label(
                        LabelData {
                            name: "eq:bar",
                            header: Some(
                                "Equation (2)",
                            ),
                            footer: None,
                            object: Some(
                                Equation,
                            ),
                            keywords: "eq:bar Equation (2)",
                        },
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 65..65,
                    preselect: false,
                    data: Label(
                        LabelData {
                            name: "eq:foo",
                            header: Some(
                                "Equation (1)",
                            ),
                            footer: None,
                            object: Some(
                                Equation,
                            ),
                            keywords: "eq:foo Equation (1)",
                        },
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 65..65,
                    preselect: false,
                    data: Label(
                        LabelData {
                            name: "fig:baz",
                            header: Some(
                                "Figure 1",
                            ),
                            footer: Some(
                                "Baz",
                            ),
                            object: Some(
                                Float {
                                    kind: Figure,
                                    caption: "Baz",
                                },
                            ),
                            keywords: "fig:baz Figure 1: Baz",
                        },
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 65..65,
                    preselect: false,
                    data: Label(
                        LabelData {
                            name: "sec:bar",
                            header: Some(
                                "Section (Bar)",
                            ),
                            footer: None,
                            object: Some(
                                Section {
                                    prefix: "Section",
                                    text: "Bar",
                                },
                            ),
                            keywords: "sec:bar Section (Bar)",
                        },
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 65..65,
                    preselect: false,
                    data: Label(
                        LabelData {
                            name: "sec:foo",
                            header: Some(
                                "Section 1 (Foo)",
                            ),
                            footer: None,
                            object: Some(
                                Section {
                                    prefix: "Section",
                                    text: "Foo",
                                },
                            ),
                            keywords: "sec:foo Section 1 (Foo)",
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn theorem_begin() {
    check(
        r#"
%! main.tex
\newtheorem{lemma}{Lemma}
\begin{lem
        |
       ^^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 71,
                    range: 33..36,
                    preselect: false,
                    data: Environment(
                        EnvironmentData {
                            name: "lemma",
                            package: "<user>",
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn theorem_end() {
    check(
        r#"
%! main.tex
\newtheorem{lemma}{Lemma}
\begin{}
\end{lem
      |
     ^^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 71,
                    range: 40..43,
                    preselect: false,
                    data: Environment(
                        EnvironmentData {
                            name: "lemma",
                            package: "<user>",
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn tikz_library_open_brace() {
    check(
        r#"
%! main.tex
\usepgflibrary{
               |"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 0,
                    range: 15..15,
                    preselect: false,
                    data: TikzLibrary(
                        "arrows",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 15..15,
                    preselect: false,
                    data: TikzLibrary(
                        "arrows.meta",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 15..15,
                    preselect: false,
                    data: TikzLibrary(
                        "arrows.spaced",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 15..15,
                    preselect: false,
                    data: TikzLibrary(
                        "curvilinear",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 15..15,
                    preselect: false,
                    data: TikzLibrary(
                        "datavisualization.barcharts",
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn tikz_library_closed_brace() {
    check(
        r#"
%! main.tex
\usepgflibrary{}
               |"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 0,
                    range: 15..15,
                    preselect: false,
                    data: TikzLibrary(
                        "arrows",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 15..15,
                    preselect: false,
                    data: TikzLibrary(
                        "arrows.meta",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 15..15,
                    preselect: false,
                    data: TikzLibrary(
                        "arrows.spaced",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 15..15,
                    preselect: false,
                    data: TikzLibrary(
                        "curvilinear",
                    ),
                },
                CompletionItem {
                    score: 0,
                    range: 15..15,
                    preselect: false,
                    data: TikzLibrary(
                        "datavisualization.barcharts",
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn test_user_command() {
    check(
        r#"
%! main.tex
\foobar
\fooba
   |
 ^^^^^
\begin{foo}
\end{foo}
\begin{fo}"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 111,
                    range: 9..14,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "foobar",
                            package: "<user>",
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn test_user_environment() {
    check(
        r#"
%! main.tex
\foobar
\fooba
\begin{foo}
\end{foo}
\begin{fo}
        |
       ^^"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 51,
                    range: 44..46,
                    preselect: false,
                    data: Environment(
                        EnvironmentData {
                            name: "foo",
                            package: "<user>",
                        },
                    ),
                },
                CompletionItem {
                    score: 40,
                    range: 44..46,
                    preselect: false,
                    data: Environment(
                        EnvironmentData {
                            name: "filecontents",
                            package: [],
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn test_project_resolution_import() {
    check(
        r#"
%! main.tex
\documentclass{article}
\import{sub}{sub.tex}
\lipsu
     |
 ^^^^^

%! sub/sub.tex
\input{child.tex}

%! sub/child.tex
\usepackage{lipsum}"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 111,
                    range: 47..52,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "lipsum",
                            package: [
                                "lipsum.sty",
                            ],
                        },
                    ),
                },
                CompletionItem {
                    score: 111,
                    range: 47..52,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "lipsumexp",
                            package: [
                                "lipsum.sty",
                            ],
                        },
                    ),
                },
                CompletionItem {
                    score: 109,
                    range: 47..52,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "LipsumPar",
                            package: [
                                "lipsum.sty",
                            ],
                        },
                    ),
                },
                CompletionItem {
                    score: 109,
                    range: 47..52,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "LipsumProtect",
                            package: [
                                "lipsum.sty",
                            ],
                        },
                    ),
                },
                CompletionItem {
                    score: 109,
                    range: 47..52,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "LipsumRestoreAll",
                            package: [
                                "lipsum.sty",
                            ],
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn test_project_resolution_texlabroot() {
    check(
        r#"
%! src/main.tex
\documentclass{article}
\include{src/foo}
\lipsu
     |
 ^^^^^

%! src/foo.tex
\include{src/bar}

%! src/bar.tex
\usepackage{lipsum}

%! .texlabroot"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 111,
                    range: 43..48,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "lipsum",
                            package: [
                                "lipsum.sty",
                            ],
                        },
                    ),
                },
                CompletionItem {
                    score: 111,
                    range: 43..48,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "lipsumexp",
                            package: [
                                "lipsum.sty",
                            ],
                        },
                    ),
                },
                CompletionItem {
                    score: 109,
                    range: 43..48,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "LipsumPar",
                            package: [
                                "lipsum.sty",
                            ],
                        },
                    ),
                },
                CompletionItem {
                    score: 109,
                    range: 43..48,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "LipsumProtect",
                            package: [
                                "lipsum.sty",
                            ],
                        },
                    ),
                },
                CompletionItem {
                    score: 109,
                    range: 43..48,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "LipsumRestoreAll",
                            package: [
                                "lipsum.sty",
                            ],
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn issue_857_1() {
    check(
        r#"
%! bug.tex
\documentclass{article}
\newcommand{\ö}{foo}
\newcommand{\öö}{bar}
\newcommand{\ööabc}{baz}
\begin{document}
\ö
  |
 ^
\end{document}
"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 31,
                    range: 115..117,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "ö",
                            package: "<user>",
                        },
                    ),
                },
                CompletionItem {
                    score: 31,
                    range: 115..117,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "öö",
                            package: "<user>",
                        },
                    ),
                },
                CompletionItem {
                    score: 31,
                    range: 115..117,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "ööabc",
                            package: "<user>",
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn issue_864() {
    check(
        r#"
%! bug.tex
\documentclass{article}
\def\あいうえお{}
\begin{document}
\あ
  |
 ^
\end{document}"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 31,
                    range: 65..68,
                    preselect: false,
                    data: Command(
                        CommandData {
                            name: "あいうえお",
                            package: "<user>",
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn issue_883() {
    check(
        r#"
%! bug.tex
\begin{doc
          |
       ^^^
% Comment"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 71,
                    range: 7..10,
                    preselect: false,
                    data: Environment(
                        EnvironmentData {
                            name: "document",
                            package: [],
                        },
                    ),
                },
            ]
        "#]],
    );
}

#[test]
fn issue_885() {
    check(
        r#"
%! main.tex
\documentclass{book}
\usepackage{import}
\begin{document}
\subincludefrom{part 1}{main}
\include{part 2/main}

\ref{sec}
      |
     ^^^
\end{document}

%! part 1/main.tex
\part{1}
\label{part 1}
\subimport{chapter 1}{main}

%! part 1/chapter 1/main.tex
\chapter{1}
\label{chapter 1}
\subimport{./}{section 1}
%\subimport{}{section 1}

%! part 1/chapter 1/section 1.tex
\section{1}
\label{section 1}

%! part 2/main.tex
\part{2}
\label{part 2}
\input{part 2/chapter 2/main}

%! part 2/chapter 2/main.tex
\chapter{2}
\label{chapter 2}
\input{part 2/chapter 2/section 2}

%! part 2/chapter 2/section 2.tex
\section{2}
\label{section 2}
"#,
        expect![[r#"
            [
                CompletionItem {
                    score: 71,
                    range: 116..119,
                    preselect: false,
                    data: Label(
                        LabelData {
                            name: "section 1",
                            header: Some(
                                "Section (1)",
                            ),
                            footer: None,
                            object: Some(
                                Section {
                                    prefix: "Section",
                                    text: "1",
                                },
                            ),
                            keywords: "section 1 Section (1)",
                        },
                    ),
                },
                CompletionItem {
                    score: 71,
                    range: 116..119,
                    preselect: false,
                    data: Label(
                        LabelData {
                            name: "section 2",
                            header: Some(
                                "Section (2)",
                            ),
                            footer: None,
                            object: Some(
                                Section {
                                    prefix: "Section",
                                    text: "2",
                                },
                            ),
                            keywords: "section 2 Section (2)",
                        },
                    ),
                },
            ]
        "#]],
    );
}
