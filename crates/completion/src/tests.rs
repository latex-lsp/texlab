use base_db::{Config, FeatureParams};
use expect_test::{expect, Expect};
use parser::SyntaxConfig;
use rowan::TextRange;

use crate::CompletionParams;

fn check_with_syntax_config(config: SyntaxConfig, input: &str, expect: Expect) {
    let mut fixture = test_utils::fixture::Fixture::parse(input);
    fixture.workspace.set_config(Config {
        syntax: config,
        ..Config::default()
    });
    let fixture = fixture;

    let (offset, spec) = fixture
        .documents
        .iter()
        .find_map(|document| Some((document.cursor?, document)))
        .unwrap();

    let document = fixture.workspace.lookup(&spec.uri).unwrap();
    let feature = FeatureParams::new(&fixture.workspace, document);
    let params = CompletionParams { feature, offset };
    let result = crate::complete(&params);

    let range = spec
        .ranges
        .first()
        .map_or_else(|| TextRange::empty(offset), |range| *range);

    for item in &result.items {
        assert_eq!(item.range, range);
    }

    let items = result
        .items
        .into_iter()
        .take(5)
        .map(|item| item.data)
        .collect::<Vec<_>>();

    expect.assert_debug_eq(&items);
}

fn check(input: &str, expect: Expect) {
    check_with_syntax_config(SyntaxConfig::default(), input, expect)
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
                GlossaryEntry(
                    GlossaryEntryData {
                        name: "fpsLabel",
                    },
                ),
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
                GlossaryEntry(
                    GlossaryEntryData {
                        name: "fpsLabel",
                    },
                ),
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
                GlossaryEntry(
                    GlossaryEntryData {
                        name: "fpsLabel",
                    },
                ),
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
                GlossaryEntry(
                    GlossaryEntryData {
                        name: "fpsLabel",
                    },
                ),
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
                GlossaryEntry(
                    GlossaryEntryData {
                        name: "fpsLabel",
                    },
                ),
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
                GlossaryEntry(
                    GlossaryEntryData {
                        name: "fpsLabel",
                    },
                ),
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
                Argument(
                    ArgumentData(
                        "A",
                    ),
                ),
                Argument(
                    ArgumentData(
                        "B",
                    ),
                ),
                Argument(
                    ArgumentData(
                        "C",
                    ),
                ),
                Argument(
                    ArgumentData(
                        "D",
                    ),
                ),
                Argument(
                    ArgumentData(
                        "E",
                    ),
                ),
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
                Argument(
                    ArgumentData(
                        "A",
                    ),
                ),
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
                Argument(
                    ArgumentData(
                        "A",
                    ),
                ),
                Argument(
                    ArgumentData(
                        "B",
                    ),
                ),
                Argument(
                    ArgumentData(
                        "C",
                    ),
                ),
                Argument(
                    ArgumentData(
                        "D",
                    ),
                ),
                Argument(
                    ArgumentData(
                        "E",
                    ),
                ),
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
                BeginEnvironment,
                Command(
                    CommandData {
                        name: "begingroup",
                        package: [],
                    },
                ),
                Command(
                    CommandData {
                        name: "AtBeginDocument",
                        package: [],
                    },
                ),
                Command(
                    CommandData {
                        name: "AtBeginDvi",
                        package: [],
                    },
                ),
                Command(
                    CommandData {
                        name: "bigwedge",
                        package: [],
                    },
                ),
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
                Citation(
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
                Citation(
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
                Citation(
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
                Citation(
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
                Citation(
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
                ColorModel(
                    "HTML",
                ),
                ColorModel(
                    "RGB",
                ),
                ColorModel(
                    "cmyk",
                ),
                ColorModel(
                    "gray",
                ),
                ColorModel(
                    "rgb",
                ),
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
                ColorModel(
                    "HTML",
                ),
                ColorModel(
                    "RGB",
                ),
                ColorModel(
                    "cmyk",
                ),
                ColorModel(
                    "gray",
                ),
                ColorModel(
                    "rgb",
                ),
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
                ColorModel(
                    "HTML",
                ),
                ColorModel(
                    "RGB",
                ),
                ColorModel(
                    "cmyk",
                ),
                ColorModel(
                    "gray",
                ),
                ColorModel(
                    "rgb",
                ),
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
                ColorModel(
                    "HTML",
                ),
                ColorModel(
                    "RGB",
                ),
                ColorModel(
                    "cmyk",
                ),
                ColorModel(
                    "gray",
                ),
                ColorModel(
                    "rgb",
                ),
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
                Color(
                    "Apricot",
                ),
                Color(
                    "Aquamarine",
                ),
                Color(
                    "Bittersweet",
                ),
                Color(
                    "Black",
                ),
                Color(
                    "Blue",
                ),
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
                Color(
                    "red",
                ),
                Color(
                    "Red",
                ),
                Color(
                    "RedOrange",
                ),
                Color(
                    "RedViolet",
                ),
                Color(
                    "BrickRed",
                ),
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
                Color(
                    "Apricot",
                ),
                Color(
                    "Aquamarine",
                ),
                Color(
                    "Bittersweet",
                ),
                Color(
                    "Black",
                ),
                Color(
                    "Blue",
                ),
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
        expect![[r#"
            [
                CommandLikeDelimiter(
                    "(",
                    ")",
                ),
                CommandLikeDelimiter(
                    "[",
                    "]",
                ),
                CommandLikeDelimiter(
                    "{",
                    "\\}",
                ),
                Command(
                    CommandData {
                        name: "!",
                        package: [],
                    },
                ),
                Command(
                    CommandData {
                        name: "\"",
                        package: [],
                    },
                ),
            ]
        "#]],
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
                Command(
                    CommandData {
                        name: "lipsum",
                        package: [
                            "lipsum.sty",
                        ],
                    },
                ),
                Command(
                    CommandData {
                        name: "lipsumexp",
                        package: [
                            "lipsum.sty",
                        ],
                    },
                ),
                Command(
                    CommandData {
                        name: "LipsumPar",
                        package: [
                            "lipsum.sty",
                        ],
                    },
                ),
                Command(
                    CommandData {
                        name: "LipsumProtect",
                        package: [
                            "lipsum.sty",
                        ],
                    },
                ),
                Command(
                    CommandData {
                        name: "LipsumRestoreAll",
                        package: [
                            "lipsum.sty",
                        ],
                    },
                ),
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
                Command(
                    CommandData {
                        name: "LaTeX",
                        package: [],
                    },
                ),
                Command(
                    CommandData {
                        name: "LaTeXe",
                        package: [],
                    },
                ),
                Command(
                    CommandData {
                        name: "latexreleaseversion",
                        package: [],
                    },
                ),
                Command(
                    CommandData {
                        name: "LastDeclaredEncoding",
                        package: [],
                    },
                ),
                Command(
                    CommandData {
                        name: "last",
                        package: [],
                    },
                ),
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
                Environment(
                    EnvironmentData {
                        name: "document",
                        package: [],
                    },
                ),
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
                Environment(
                    EnvironmentData {
                        name: "document",
                        package: [],
                    },
                ),
                Environment(
                    EnvironmentData {
                        name: "abstract",
                        package: [],
                    },
                ),
                Environment(
                    EnvironmentData {
                        name: "array",
                        package: [],
                    },
                ),
                Environment(
                    EnvironmentData {
                        name: "center",
                        package: [],
                    },
                ),
                Environment(
                    EnvironmentData {
                        name: "csname",
                        package: [],
                    },
                ),
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
                Environment(
                    EnvironmentData {
                        name: "theindex",
                        package: [
                            "article.cls",
                        ],
                    },
                ),
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
                Environment(
                    EnvironmentData {
                        name: "document",
                        package: [],
                    },
                ),
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
 |
^"#,
        expect![[r#"
            [
                EntryType(
                    EntryTypeData(
                        "@article",
                    ),
                ),
                EntryType(
                    EntryTypeData(
                        "@artwork",
                    ),
                ),
                EntryType(
                    EntryTypeData(
                        "@audio",
                    ),
                ),
                EntryType(
                    EntryTypeData(
                        "@bibnote",
                    ),
                ),
                EntryType(
                    EntryTypeData(
                        "@book",
                    ),
                ),
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
^^^^^^^^^"#,
        expect![[r#"
            [
                EntryType(
                    EntryTypeData(
                        "@preamble",
                    ),
                ),
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
^^^^^^^"#,
        expect![[r#"
            [
                EntryType(
                    EntryTypeData(
                        "@string",
                    ),
                ),
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
^^^^^^^^"#,
        expect![[r#"
            [
                EntryType(
                    EntryTypeData(
                        "@article",
                    ),
                ),
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
^^^^^^^^^"#,
        expect![[r#"
            [
                EntryType(
                    EntryTypeData(
                        "@preamble",
                    ),
                ),
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
^^^^^^^"#,
        expect![[r#"
            [
                EntryType(
                    EntryTypeData(
                        "@string",
                    ),
                ),
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
^^^^^^^^"#,
        expect![[r#"
            [
                EntryType(
                    EntryTypeData(
                        "@article",
                    ),
                ),
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
                Field(
                    FieldTypeData(
                        "abstract",
                    ),
                ),
                Field(
                    FieldTypeData(
                        "addendum",
                    ),
                ),
                Field(
                    FieldTypeData(
                        "address",
                    ),
                ),
                Field(
                    FieldTypeData(
                        "afterword",
                    ),
                ),
                Field(
                    FieldTypeData(
                        "annotation",
                    ),
                ),
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
                Field(
                    FieldTypeData(
                        "abstract",
                    ),
                ),
                Field(
                    FieldTypeData(
                        "addendum",
                    ),
                ),
                Field(
                    FieldTypeData(
                        "address",
                    ),
                ),
                Field(
                    FieldTypeData(
                        "afterword",
                    ),
                ),
                Field(
                    FieldTypeData(
                        "annotation",
                    ),
                ),
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
                Field(
                    FieldTypeData(
                        "abstract",
                    ),
                ),
                Field(
                    FieldTypeData(
                        "addendum",
                    ),
                ),
                Field(
                    FieldTypeData(
                        "address",
                    ),
                ),
                Field(
                    FieldTypeData(
                        "afterword",
                    ),
                ),
                Field(
                    FieldTypeData(
                        "annotation",
                    ),
                ),
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
                Field(
                    FieldTypeData(
                        "edition",
                    ),
                ),
                Field(
                    FieldTypeData(
                        "editor",
                    ),
                ),
                Field(
                    FieldTypeData(
                        "editora",
                    ),
                ),
                Field(
                    FieldTypeData(
                        "editoratype",
                    ),
                ),
                Field(
                    FieldTypeData(
                        "editorb",
                    ),
                ),
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
                Field(
                    FieldTypeData(
                        "edition",
                    ),
                ),
                Field(
                    FieldTypeData(
                        "editor",
                    ),
                ),
                Field(
                    FieldTypeData(
                        "editora",
                    ),
                ),
                Field(
                    FieldTypeData(
                        "editoratype",
                    ),
                ),
                Field(
                    FieldTypeData(
                        "editorb",
                    ),
                ),
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
                Package(
                    "lips",
                ),
                Package(
                    "lipsum",
                ),
                Package(
                    "lisp-simple-alloc",
                ),
                Package(
                    "lisp-string",
                ),
                Package(
                    "lwarp-lips",
                ),
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
                Package(
                    "lips",
                ),
                Package(
                    "lipsum",
                ),
                Package(
                    "lisp-simple-alloc",
                ),
                Package(
                    "lisp-string",
                ),
                Package(
                    "lwarp-lips",
                ),
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
                DocumentClass(
                    "article",
                ),
                DocumentClass(
                    "articleingud",
                ),
                DocumentClass(
                    "articoletteracdp",
                ),
                DocumentClass(
                    "artikel1",
                ),
                DocumentClass(
                    "artikel2",
                ),
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
                DocumentClass(
                    "article",
                ),
                DocumentClass(
                    "articleingud",
                ),
                DocumentClass(
                    "articoletteracdp",
                ),
                DocumentClass(
                    "artikel1",
                ),
                DocumentClass(
                    "artikel2",
                ),
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
                Label(
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
                Label(
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
                Label(
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
                Label(
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
                Label(
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
            ]
        "#]],
    );
}

#[test]
fn label_undefined() {
    check(
        r#"
%! foo.tex
\label{f}
        |
       ^
\ref{foo}"#,
        expect![[r#"
            [
                Label(
                    LabelData {
                        name: "foo",
                        header: None,
                        footer: None,
                        object: None,
                        keywords: "foo",
                    },
                ),
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
                Environment(
                    EnvironmentData {
                        name: "lemma",
                        package: "<user>",
                    },
                ),
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
                Environment(
                    EnvironmentData {
                        name: "lemma",
                        package: "<user>",
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn theorem_begin_multiple() {
    check(
        r#"
%! main.tex
\declaretheorem[sibling=table, style=thmbox]{def1, def2, def3, def4, def5}
\begin{def
        |
       ^^^"#,
        expect![[r#"
            [
                Environment(
                    EnvironmentData {
                        name: "def1",
                        package: "<user>",
                    },
                ),
                Environment(
                    EnvironmentData {
                        name: "def2",
                        package: "<user>",
                    },
                ),
                Environment(
                    EnvironmentData {
                        name: "def3",
                        package: "<user>",
                    },
                ),
                Environment(
                    EnvironmentData {
                        name: "def4",
                        package: "<user>",
                    },
                ),
                Environment(
                    EnvironmentData {
                        name: "def5",
                        package: "<user>",
                    },
                ),
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
                TikzLibrary(
                    "arrows",
                ),
                TikzLibrary(
                    "arrows.meta",
                ),
                TikzLibrary(
                    "arrows.spaced",
                ),
                TikzLibrary(
                    "curvilinear",
                ),
                TikzLibrary(
                    "datavisualization.barcharts",
                ),
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
                TikzLibrary(
                    "arrows",
                ),
                TikzLibrary(
                    "arrows.meta",
                ),
                TikzLibrary(
                    "arrows.spaced",
                ),
                TikzLibrary(
                    "curvilinear",
                ),
                TikzLibrary(
                    "datavisualization.barcharts",
                ),
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
                Command(
                    CommandData {
                        name: "foobar",
                        package: "<user>",
                    },
                ),
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
                Environment(
                    EnvironmentData {
                        name: "foo",
                        package: "<user>",
                    },
                ),
                Environment(
                    EnvironmentData {
                        name: "filecontents",
                        package: [],
                    },
                ),
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
                Command(
                    CommandData {
                        name: "lipsum",
                        package: [
                            "lipsum.sty",
                        ],
                    },
                ),
                Command(
                    CommandData {
                        name: "lipsumexp",
                        package: [
                            "lipsum.sty",
                        ],
                    },
                ),
                Command(
                    CommandData {
                        name: "LipsumPar",
                        package: [
                            "lipsum.sty",
                        ],
                    },
                ),
                Command(
                    CommandData {
                        name: "LipsumProtect",
                        package: [
                            "lipsum.sty",
                        ],
                    },
                ),
                Command(
                    CommandData {
                        name: "LipsumRestoreAll",
                        package: [
                            "lipsum.sty",
                        ],
                    },
                ),
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
                Command(
                    CommandData {
                        name: "lipsum",
                        package: [
                            "lipsum.sty",
                        ],
                    },
                ),
                Command(
                    CommandData {
                        name: "lipsumexp",
                        package: [
                            "lipsum.sty",
                        ],
                    },
                ),
                Command(
                    CommandData {
                        name: "LipsumPar",
                        package: [
                            "lipsum.sty",
                        ],
                    },
                ),
                Command(
                    CommandData {
                        name: "LipsumProtect",
                        package: [
                            "lipsum.sty",
                        ],
                    },
                ),
                Command(
                    CommandData {
                        name: "LipsumRestoreAll",
                        package: [
                            "lipsum.sty",
                        ],
                    },
                ),
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
                Command(
                    CommandData {
                        name: "ö",
                        package: "<user>",
                    },
                ),
                Command(
                    CommandData {
                        name: "öö",
                        package: "<user>",
                    },
                ),
                Command(
                    CommandData {
                        name: "ööabc",
                        package: "<user>",
                    },
                ),
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
                Command(
                    CommandData {
                        name: "あいうえお",
                        package: "<user>",
                    },
                ),
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
                Environment(
                    EnvironmentData {
                        name: "document",
                        package: [],
                    },
                ),
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
                Label(
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
                Label(
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
            ]
        "#]],
    );
}

#[test]
fn test_custom_label_prefix_ref() {
    let mut config = SyntaxConfig::default();
    config.label_definition_commands.insert("asm".to_string());
    config.label_reference_commands.insert("asmref".to_string());
    config
        .label_definition_prefixes
        .insert("asm".to_string(), "asm:".to_string());
    config
        .label_reference_prefixes
        .insert("asm".to_string(), "asm:".to_string());

    check_with_syntax_config(
        config,
        r#"
%! main.tex
\documentclass{article}
\newcommand{\asm}[2]{\item\label[asm]{asm:#1} {#2}}
\newcommand{\asmref}[1]{\ref{asm:#1}}
\begin{document}
    \begin{enumerate}\label{baz}
        \asm{foo}{what}
    \end{enumerate}

    \ref{}
         |
\end{document}
% Comment"#,
        expect![[r#"
            [
                Label(
                    LabelData {
                        name: "asm:foo",
                        header: None,
                        footer: None,
                        object: None,
                        keywords: "asm:foo",
                    },
                ),
                Label(
                    LabelData {
                        name: "baz",
                        header: None,
                        footer: None,
                        object: None,
                        keywords: "baz",
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn test_custom_label_prefix_custom_ref() {
    let mut config = SyntaxConfig::default();
    config.label_definition_commands.insert("asm".to_string());
    config.label_reference_commands.insert("asmref".to_string());
    config
        .label_reference_prefixes
        .insert("asm".to_string(), "asm:".to_string());

    check_with_syntax_config(
        config,
        r#"
%! main.tex
\documentclass{article}
\newcommand{\asm}[2]{\item\label[asm]{asm:#1} {#2}}
\newcommand{\asmref}[1]{\ref{asm:#1}}
\begin{document}
    \begin{enumerate}\label{baz}
        \asm{foo}{what}
    \end{enumerate}

    \asmref{}
            |
\end{document}
% Comment"#,
        expect![[r#"
            [
                Label(
                    LabelData {
                        name: "foo",
                        header: None,
                        footer: None,
                        object: None,
                        keywords: "foo",
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn test_custom_label_multiple_prefix_custom_ref() {
    let mut config = SyntaxConfig::default();
    config
        .label_definition_commands
        .extend(vec!["asm", "goal"].into_iter().map(String::from));
    config.label_definition_prefixes.extend(
        vec![("asm", "asm:"), ("goal", "goal:")]
            .into_iter()
            .map(|(x, y)| (String::from(x), String::from(y))),
    );
    config
        .label_reference_commands
        .extend(vec!["asmref", "goalref"].into_iter().map(String::from));
    config.label_reference_prefixes.extend(
        vec![("asmref", "asm:"), ("goalref", "goal:")]
            .into_iter()
            .map(|(x, y)| (String::from(x), String::from(y))),
    );

    check_with_syntax_config(
        config,
        r#"
%! main.tex
\documentclass{article}
\newcommand{\asm}[2]{\item\label[asm]{asm:#1} {#2}}
\newcommand{\asmref}[1]{\ref{asm:#1}}
\newcommand{\goal}[2]{\item\label[goal]{goal:#1} {#2}}
\newcommand{\goalref}[1]{\ref{goal:#1}}
\begin{document}
    \begin{enumerate}\label{baz}
        \asm{foo}{what}
        \goal{foo}{what}
    \end{enumerate}

    \goalref{}
             |
\end{document}
% Comment"#,
        expect![[r#"
            [
                Label(
                    LabelData {
                        name: "foo",
                        header: None,
                        footer: None,
                        object: None,
                        keywords: "foo",
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn test_custom_label_multiple_prefix_ref() {
    let mut config = SyntaxConfig::default();
    config
        .label_definition_commands
        .extend(vec!["asm", "goal"].into_iter().map(String::from));
    config.label_definition_prefixes.extend(
        vec![("asm", "asm:"), ("goal", "goal:")]
            .into_iter()
            .map(|(x, y)| (String::from(x), String::from(y))),
    );
    config
        .label_reference_commands
        .extend(vec!["asmref", "goalref"].into_iter().map(String::from));
    config.label_reference_prefixes.extend(
        vec![("asmref", "asm:"), ("goalref", "goal:")]
            .into_iter()
            .map(|(x, y)| (String::from(x), String::from(y))),
    );

    check_with_syntax_config(
        config,
        r#"
%! main.tex
\documentclass{article}
\newcommand{\asm}[2]{\item\label[asm]{asm:#1} {#2}}
\newcommand{\asmref}[1]{\ref{asm:#1}}
\newcommand{\goal}[2]{\item\label[goal]{goal:#1} {#2}}
\newcommand{\goalref}[1]{\ref{goal:#1}}
\begin{document}
    \begin{enumerate}\label{baz}
        \asm{foo}{what}
        \goal{foo}{what}
    \end{enumerate}

    \ref{}
         |
\end{document}
% Comment"#,
        expect![[r#"
            [
                Label(
                    LabelData {
                        name: "asm:foo",
                        header: None,
                        footer: None,
                        object: None,
                        keywords: "asm:foo",
                    },
                ),
                Label(
                    LabelData {
                        name: "baz",
                        header: None,
                        footer: None,
                        object: None,
                        keywords: "baz",
                    },
                ),
                Label(
                    LabelData {
                        name: "goal:foo",
                        header: None,
                        footer: None,
                        object: None,
                        keywords: "goal:foo",
                    },
                ),
            ]
        "#]],
    );
}
