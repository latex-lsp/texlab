use anyhow::Result;
use insta::assert_json_snapshot;
use lsp_types::{
    request::{Completion, ResolveCompletionItem},
    ClientCapabilities, CompletionItem, CompletionList, CompletionParams, CompletionResponse,
    CompletionTextEdit, Range,
};

use crate::{client::Client, fixture};

fn complete(fixture: &str) -> Result<Vec<CompletionItem>, anyhow::Error> {
    let mut client = Client::spawn()?;
    client.initialize(ClientCapabilities::default(), None)?;
    let fixture = fixture::parse(fixture);
    for file in fixture.files {
        client.open(file.name, file.lang, file.text)?;
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
            text_document_position: fixture.cursor.unwrap().into_params(&client)?,
            partial_result_params: Default::default(),
            work_done_progress_params: Default::default(),
            context: None,
        })?
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
        .map(|item| client.request::<ResolveCompletionItem>(item))
        .collect::<Result<Vec<_>>>()?;

    client.shutdown()?;
    Ok(actual_items)
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
fn acronym_ref_simple() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
%SRC \acrshort{f}
%CUR           ^
%1.1           ^"#
    )?);

    Ok(())
}

#[test]
fn acronym_ref_empty() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
%SRC \acrshort{}
%CUR           ^"#
    )?);

    Ok(())
}

#[test]
fn acronym_ref_after_group() -> Result<()> {
    let actual_items = complete(
        r#"
%TEX main.tex
%SRC \newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
%SRC \acrshort{}
%CUR            ^
%1.1            ^"#,
    )?;

    assert_eq!(actual_items, Vec::new());
    Ok(())
}

#[test]
fn acronym_ref_open_brace() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
%SRC \acrshort{f
%CUR           ^
%1.1           ^"#
    )?);

    Ok(())
}

#[test]
fn glossary_ref_simple() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
%SRC \gls{f}
%CUR      ^
%1.1      ^"#
    )?);

    Ok(())
}

#[test]
fn glossary_ref_open_brace() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}
%SRC \gls{f
%CUR      ^
%1.1      ^"#
    )?);

    Ok(())
}

#[test]
fn argument_empty() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \usepackage{amsfonts}
%SRC \mathbb{}
%CUR         ^"#
    )?);

    Ok(())
}

#[test]
fn argument_word() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \usepackage{amsfonts}
%SRC \mathbb{A}
%CUR         ^
%1.1         ^"#
    )?);

    Ok(())
}

#[test]
fn argument_open_brace() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \usepackage{amsfonts}
%SRC \mathbb{
%CUR         ^
%SRC Test"#
    )?);

    Ok(())
}

#[test]
fn argument_open_brace_unrelated() -> Result<()> {
    let actual_items = complete(
        r#"
%TEX main.tex
%SRC \usepackage{amsfonts}
%SRC \mathbb{}{
%CUR           ^
%SRC Test"#,
    )?;

    assert_eq!(actual_items, Vec::new());
    Ok(())
}

#[test]
fn begin_environment_without_snippet_support() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \beg
%CUR     ^
%1.1  ^^^"#
    )?);

    Ok(())
}

#[test]
fn citation() -> Result<()> {
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
    )?);

    Ok(())
}

#[test]
fn citation_open_brace() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \addbibresource{main.bib}
%SRC \cite{
%CUR       ^

%BIB main.bib
%SRC @article{foo,}"#
    )?);

    Ok(())
}

#[test]
fn citation_open_brace_multiple() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \addbibresource{main.bib}
%SRC \cite{foo,a
%CUR           ^
%1.1           ^

%BIB main.bib
%SRC @article{foo,}"#
    )?);

    Ok(())
}

#[test]
fn citation_acronym() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \addbibresource{main.bib}
%SRC \DeclareAcronym{foo}{cite={}}
%CUR                            ^

%BIB main.bib
%SRC @article{foo,}"#
    )?);

    Ok(())
}

#[test]
fn citation_after_brace() -> Result<()> {
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
    )?;

    assert_eq!(actual_items, Vec::new());
    Ok(())
}

#[test]
fn color_model_definition_simple() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \definecolor{foo}{}
%CUR                   ^"#
    )?);

    Ok(())
}

#[test]
fn color_model_definition_open_brace() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \definecolor{foo}{
%CUR                   ^"#
    )?);

    Ok(())
}

#[test]
fn color_model_definition_set_simple() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \definecolorset{}
%CUR                 ^"#
    )?);

    Ok(())
}

#[test]
fn color_model_definition_set_open_brace() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \definecolorset{
%CUR                 ^"#
    )?);

    Ok(())
}

#[test]
fn color_simple() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \color{}
%CUR        ^"#
    )?);

    Ok(())
}

#[test]
fn color_word() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \color{re}
%CUR         ^
%1.1        ^^"#
    )?);

    Ok(())
}

#[test]
fn color_open_brace() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \color{
%CUR        ^"#
    )?);

    Ok(())
}

#[test]
fn component_command_simple() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \
%CUR  ^"#
    )?);

    Ok(())
}

#[test]
fn component_command_simple_before() -> Result<()> {
    let actual_items = complete(
        r#"
%TEX main.tex
%SRC \
%CUR ^"#,
    )?;

    assert_eq!(actual_items, Vec::new());
    Ok(())
}

#[test]
fn component_command_simple_package() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \usepackage{lipsum}
%SRC \lips
%CUR    ^
%1.1  ^^^^"#
    )?);

    Ok(())
}

#[test]
fn component_command_bibtex() -> Result<()> {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @article{b, 
%SRC     c = {\LaT }
%CUR            ^
%1.1           ^^^
%SRC }"#
    )?);

    Ok(())
}

#[test]
fn component_environment_simple() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \begin{doc
%CUR           ^
%1.1        ^^^"#
    )?);

    Ok(())
}

#[test]
fn component_environment_simple_end() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \begin{document}
%SRC \end{
%CUR      ^"#
    )?);

    Ok(())
}

#[test]
fn component_environment_class() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \documentclass{article}
%SRC \begin{thein}
%CUR           ^
%1.1        ^^^^^"#
    )?);

    Ok(())
}

#[test]
fn component_environment_command_definition() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \newcommand{\foo}{\begin{doc}
%CUR                            ^
%1.1                          ^^^"#
    )?);

    Ok(())
}

#[test]
fn entry_type_at_empty() -> Result<()> {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @
%CUR  ^"#
    )?);

    Ok(())
}

#[test]
fn entry_type_before_preamble() -> Result<()> {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @preamble
%CUR  ^
%1.1  ^^^^^^^^"#
    )?);

    Ok(())
}

#[test]
fn entry_type_before_string() -> Result<()> {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @string
%CUR  ^
%1.1  ^^^^^^"#
    )?);

    Ok(())
}

#[test]
fn entry_type_before_article() -> Result<()> {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @article
%CUR  ^
%1.1  ^^^^^^^"#
    )?);

    Ok(())
}

#[test]
fn entry_type_after_preamble() -> Result<()> {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @preamble{
%CUR          ^
%1.1  ^^^^^^^^"#
    )?);

    Ok(())
}

#[test]
fn entry_type_after_string() -> Result<()> {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @string{
%CUR        ^
%1.1  ^^^^^^"#
    )?);

    Ok(())
}

#[test]
fn entry_type_complete_entry() -> Result<()> {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @article{foo, author = {foo}}
%CUR    ^
%1.1  ^^^^^^^"#
    )?);

    Ok(())
}

#[test]
fn field_empty_entry_open() -> Result<()> {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @article{foo,
%CUR              ^"#
    )?);

    Ok(())
}

#[test]
fn field_empty_entry_closed() -> Result<()> {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @article{foo,}
%CUR              ^"#
    )?);

    Ok(())
}

#[test]
fn field_entry_field_name() -> Result<()> {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @article{foo, a
%CUR                ^
%1.1               ^"#
    )?);

    Ok(())
}

#[test]
fn field_entry_two_fields_name_open() -> Result<()> {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @article{foo, author = bar, edit
%CUR                              ^
%1.1                             ^^^^"#
    )?);

    Ok(())
}

#[test]
fn field_entry_two_fields_name_closed() -> Result<()> {
    assert_items!(complete(
        r#"
%BIB main.bib
%SRC @article{foo, author = bar, edit}
%CUR                              ^
%1.1                             ^^^^"#
    )?);

    Ok(())
}

#[test]
fn import_package_open_brace() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \usepackage{lips
%CUR              ^
%1.1             ^^^^"#
    )?);

    Ok(())
}

#[test]
fn import_package_closed_brace() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \usepackage{lips}
%CUR              ^
%1.1             ^^^^"#
    )?);

    Ok(())
}

#[test]
fn import_class_open_brace() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \documentclass{art \foo
%CUR                 ^
%1.1                ^^^"#
    )?);

    Ok(())
}

#[test]
fn import_class_closed_brace() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \documentclass{art}
%CUR                 ^
%1.1                ^^^"#
    )?);

    Ok(())
}

#[test]
fn label() -> Result<()> {
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
    )?);

    Ok(())
}

#[test]
fn theorem_begin() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \newtheorem{lemma}{Lemma}
%SRC \begin{lem 
%CUR         ^
%1.1        ^^^"#
    )?);

    Ok(())
}

#[test]
fn theorem_end() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \newtheorem{lemma}{Lemma}
%SRC \begin{}
%SRC \end{lem
%CUR       ^
%1.1      ^^^"#
    )?);

    Ok(())
}

#[test]
fn tikz_library_open_brace() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \usepgflibrary{
%CUR                ^"#
    )?);

    Ok(())
}

#[test]
fn tikz_library_closed_brace() -> Result<()> {
    assert_items!(complete(
        r#"
%TEX main.tex
%SRC \usepgflibrary{}
%CUR                ^"#
    )?);

    Ok(())
}

#[test]
fn test_user_command() -> Result<()> {
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
    )?);

    Ok(())
}

#[test]
fn test_user_environment() -> Result<()> {
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
    )?);

    Ok(())
}
