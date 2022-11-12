use insta::assert_json_snapshot;
use lsp_types::{
    request::{Completion, ResolveCompletionItem},
    CompletionItem, CompletionList, CompletionParams, CompletionResponse, CompletionTextEdit,
    Range,
};

use crate::tests::{client::Client, fixture};

fn complete(fixture: &str) -> Vec<CompletionItem> {
    let mut client = Client::spawn();
    client.initialize(
        serde_json::from_value(serde_json::json!({
            "textDocument": {
                "completion": {
                    "completionItem": {
                        "documentationFormat": ["plaintext", "markdown"]
                    }
                }
            }
        }))
        .unwrap(),
        None,
    );

    let fixture = fixture::parse(fixture);
    for file in fixture.files {
        client.open(file.name, file.lang, file.text);
    }

    let range = fixture
        .ranges
        .values()
        .next()
        .and_then(|map| map.values().next())
        .map_or_else(
            || {
                let position = fixture.cursor.unwrap().position;
                Range::new(position, position)
            },
            |file_range| file_range.range,
        );

    let actual_list = client
        .request::<Completion>(CompletionParams {
            text_document_position: fixture.cursor.unwrap().into_params(&client),
            partial_result_params: Default::default(),
            work_done_progress_params: Default::default(),
            context: None,
        })
        .unwrap()
        .map_or(CompletionList::default(), |actual| match actual {
            CompletionResponse::List(list) => list,
            CompletionResponse::Array(_) => unreachable!(),
        });

    for item in &actual_list.items {
        if let Some(CompletionTextEdit::Edit(edit)) = item.text_edit.as_ref() {
            assert_eq!(edit.range, range);
        }
    }

    let actual_items = actual_list
        .items
        .into_iter()
        .take(5)
        .map(|item| client.request::<ResolveCompletionItem>(item).unwrap())
        .collect();

    client.shutdown();
    actual_items
}

macro_rules! assert_items {
    ($items:expr) => {
        assert_json_snapshot!($items, {
            "[].data" => "[data]",
            "[].sortText" => "[sortText]",
            "[].documentation" => "[documentation]",
            "[].textEdit.range" => "[range]",
            "[]" => insta::sorted_redaction()
        });
    };
}

#[test]
fn acronym_ref_simple() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
%SRC \acrshort{f}
%CUR           ^
%1.1           ^"#
    ));
}

#[test]
fn acronym_ref_empty() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
%SRC \acrshort{}
%CUR           ^"#
    ));
}

#[test]
fn acronym_ref_after_group() {
    let actual_items = complete(
        r#"
%TEX main.tex
%SRC \newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
%SRC \acrshort{}
%CUR            ^
%1.1            ^"#,
    );

    assert_eq!(actual_items, Vec::new());
}

#[test]
fn acronym_ref_open_brace() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
%SRC \acrshort{f
%CUR           ^
%1.1           ^"#
    ));
}

#[test]
fn acronym_package_ref() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \acrodef{fpsLabel}[FPS]{Frames per Second}
%SRC \ac{f
%CUR     ^
%1.1     ^"#
    ));
}

#[test]
fn glossary_ref_simple() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
%SRC \gls{f}
%CUR      ^
%1.1      ^"#
    ));
}

#[test]
fn glossary_ref_open_brace() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
%SRC \gls{f
%CUR      ^
%1.1      ^"#
    ));
}

#[test]
fn argument_empty() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \usepackage{amsfonts}
%SRC \mathbb{}
%CUR         ^"#
    ));
}

#[test]
fn argument_word() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \usepackage{amsfonts}
%SRC \mathbb{A}
%CUR         ^
%1.1         ^"#
    ));
}

#[test]
fn argument_open_brace() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \usepackage{amsfonts}
%SRC \mathbb{
%CUR         ^
%SRC Test"#
    ));
}

#[test]
fn argument_open_brace_unrelated() {
    let actual_items = complete(
        r#"
%TEX main.tex
%SRC \usepackage{amsfonts}
%SRC \mathbb{}{
%CUR           ^
%SRC Test"#,
    );

    assert_eq!(actual_items, Vec::new());
}

#[test]
fn begin_environment_without_snippet_support() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \beg
%CUR     ^
%1.1  ^^^"#
    ));
}

#[test]
fn citation() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \documentclass{article}
%SRC \bibliography{main}
%SRC \begin{document}
%SRC \cite{
%CUR       ^
%SRC \end{document}

%BIB main.bib
%SRC @article{foo:2019,
%SRC     author = {Foo Bar},
%SRC     title = {Baz Qux},
%SRC     year = {2019},
%SRC }
%SRC 
%SRC @article{bar:2005,}"#
    ));
}

#[test]
fn citation_open_brace() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \addbibresource{main.bib}
%SRC \cite{
%CUR       ^

%BIB main.bib
%SRC @article{foo,}"#
    ));
}

#[test]
fn citation_open_brace_multiple() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \addbibresource{main.bib}
%SRC \cite{foo,a
%CUR           ^
%1.1           ^

%BIB main.bib
%SRC @article{foo,}"#
    ));
}

#[test]
fn citation_acronym() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \addbibresource{main.bib}
%SRC \DeclareAcronym{foo}{cite={}}
%CUR                            ^

%BIB main.bib
%SRC @article{foo,}"#
    ));
}

#[test]
fn citation_after_brace() {
    let actual_items = complete(
        r#"
%TEX main.tex
%SRC \documentclass{article}
%SRC \bibliography{main}
%SRC \begin{document}
%SRC \cite{}
%CUR        ^
%SRC \end{document}

%BIB main.bib
%SRC @article{foo,}"#,
    );

    assert_eq!(actual_items, Vec::new());
}

#[test]
fn color_model_definition_simple() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \definecolor{foo}{}
%CUR                   ^"#
    ));
}

#[test]
fn color_model_definition_open_brace() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \definecolor{foo}{
%CUR                   ^"#
    ));
}

#[test]
fn color_model_definition_set_simple() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \definecolorset{}
%CUR                 ^"#
    ));
}

#[test]
fn color_model_definition_set_open_brace() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \definecolorset{
%CUR                 ^"#
    ));
}

#[test]
fn color_simple() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \color{}
%CUR        ^"#
    ));
}

#[test]
fn color_word() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \color{re}
%CUR         ^
%1.1        ^^"#
    ));
}

#[test]
fn color_open_brace() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \color{
%CUR        ^"#
    ));
}

#[test]
fn component_command_simple() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \
%CUR  ^"#
    ));
}

#[test]
fn component_command_simple_before() {
    let actual_items = complete(
        r#"
%TEX main.tex
%SRC \
%CUR ^"#,
    );

    assert_eq!(actual_items, Vec::new());
}

#[test]
fn component_command_simple_package() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \usepackage{lipsum}
%SRC \lips
%CUR    ^
%1.1  ^^^^"#
    ));
}

#[test]
fn component_command_bibtex() {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @article{b, 
%SRC     c = {\LaT }
%CUR            ^
%1.1           ^^^
%SRC }"#
    ));
}

#[test]
fn component_environment_simple() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \begin{doc
%CUR           ^
%1.1        ^^^"#
    ));
}

#[test]
fn component_environment_simple_end() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \begin{document}
%SRC \end{
%CUR      ^"#
    ));
}

#[test]
fn component_environment_class() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \documentclass{article}
%SRC \begin{thein}
%CUR           ^
%1.1        ^^^^^"#
    ));
}

#[test]
fn component_environment_command_definition() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \newcommand{\foo}{\begin{doc}
%CUR                            ^
%1.1                          ^^^"#
    ));
}

#[test]
fn entry_type_at_empty() {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @
%CUR  ^"#
    ));
}

#[test]
fn entry_type_before_preamble() {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @preamble
%CUR  ^
%1.1  ^^^^^^^^"#
    ));
}

#[test]
fn entry_type_before_string() {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @string
%CUR  ^
%1.1  ^^^^^^"#
    ));
}

#[test]
fn entry_type_before_article() {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @article
%CUR  ^
%1.1  ^^^^^^^"#
    ));
}

#[test]
fn entry_type_after_preamble() {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @preamble{
%CUR          ^
%1.1  ^^^^^^^^"#
    ));
}

#[test]
fn entry_type_after_string() {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @string{
%CUR        ^
%1.1  ^^^^^^"#
    ));
}

#[test]
fn entry_type_complete_entry() {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @article{foo, author = {foo}}
%CUR    ^
%1.1  ^^^^^^^"#
    ));
}

#[test]
fn field_empty_entry_open() {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @article{foo,
%CUR              ^"#
    ));
}

#[test]
fn field_empty_entry_closed() {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @article{foo,}
%CUR              ^"#
    ));
}

#[test]
fn field_entry_field_name() {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @article{foo, a
%CUR                ^
%1.1               ^"#
    ));
}

#[test]
fn field_entry_two_fields_name_open() {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @article{foo, author = bar, edit
%CUR                              ^
%1.1                             ^^^^"#
    ));
}

#[test]
fn field_entry_two_fields_name_closed() {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @article{foo, author = bar, edit}
%CUR                              ^
%1.1                             ^^^^"#
    ));
}

#[test]
fn import_package_open_brace() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \usepackage{lips
%CUR              ^
%1.1             ^^^^"#
    ));
}

#[test]
fn import_package_closed_brace() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \usepackage{lips}
%CUR              ^
%1.1             ^^^^"#
    ));
}

#[test]
fn import_class_open_brace() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \documentclass{art \foo
%CUR                 ^
%1.1                ^^^"#
    ));
}

#[test]
fn import_class_closed_brace() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \documentclass{art}
%CUR                 ^
%1.1                ^^^"#
    ));
}

#[test]
fn label() {
    assert_items!(complete(
        r#"
%TEX foo.tex
%SRC \documentclass{article}
%SRC 
%SRC \usepackage{amsmath}
%SRC \usepackage{caption}
%SRC \usepackage{amsthm}
%SRC \newtheorem{lemma}{Lemma}
%SRC 
%SRC \begin{document}
%SRC 
%SRC \section{Foo}%
%SRC \label{sec:foo}
%SRC 
%SRC \begin{equation}%
%SRC \label{eq:foo}
%SRC     1 + 1 = 2
%SRC \end{equation}
%SRC 
%SRC \begin{equation}%
%SRC \label{eq:bar}
%SRC     1 + 1 = 2
%SRC \end{equation}
%SRC 
%SRC \begin{figure}%
%SRC \LaTeX{}
%SRC \caption{Baz}%
%SRC \label{fig:baz}
%SRC \end{figure}
%SRC 
%SRC \begin{lemma}%
%SRC \label{thm:foo}
%SRC     1 + 1 = 2
%SRC \end{lemma}
%SRC 
%SRC \include{bar}
%SRC 
%SRC \end{document}

%TEX bar.tex
%SRC \section{Bar}%
%SRC \label{sec:bar}
%SRC 
%SRC Lorem ipsum dolor sit amet.
%SRC \ref{}
%CUR      ^

%TEX foo.aux
%SRC \relax
%SRC \@writefile{lof}{\contentsline {figure}{\numberline {1}{\ignorespaces Baz\relax }}{1}\protected@file@percent }
%SRC \providecommand*\caption@xref[2]{\@setref\relax\@undefined{#1}}
%SRC \newlabel{fig:baz}{{1}{1}}
%SRC \@writefile{toc}{\contentsline {section}{\numberline {1}Foo}{1}\protected@file@percent }
%SRC \newlabel{sec:foo}{{1}{1}}
%SRC \newlabel{eq:foo}{{1}{1}}
%SRC \newlabel{eq:bar}{{2}{1}}
%SRC \newlabel{thm:foo}{{1}{1}}
%SRC \@input{bar.aux}"#
    ));
}

#[test]
fn theorem_begin() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \newtheorem{lemma}{Lemma}
%SRC \begin{lem 
%CUR         ^
%1.1        ^^^"#
    ));
}

#[test]
fn theorem_end() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \newtheorem{lemma}{Lemma}
%SRC \begin{}
%SRC \end{lem
%CUR       ^
%1.1      ^^^"#
    ));
}

#[test]
fn tikz_library_open_brace() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \usepgflibrary{
%CUR                ^"#
    ));
}

#[test]
fn tikz_library_closed_brace() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \usepgflibrary{}
%CUR                ^"#
    ));
}

#[test]
fn test_user_command() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \foobar
%SRC \fooba
%CUR    ^
%1.1  ^^^^^
%SRC \begin{foo}
%SRC \end{foo}
%SRC \begin{fo}
"#
    ));
}

#[test]
fn test_user_environment() {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \foobar
%SRC \fooba
%SRC \begin{foo}
%SRC \end{foo}
%SRC \begin{fo}
%CUR         ^
%1.1        ^^
"#
    ));
}
