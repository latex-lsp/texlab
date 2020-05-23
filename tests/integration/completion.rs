#[cfg(feature = "citation")]
use texlab::protocol::{MarkupContent, MarkupKind};

use indoc::indoc;
use itertools::Itertools;
use texlab::{
    protocol::{CompletionItem, CompletionTextEdit, Documentation, Range, RangeExt, TextEdit},
    test::{TestBed, TestBedBuilder, TestLspClient, PULL_CAPABILITIES},
};

async fn run_item(
    test_bed: &TestBed,
    relative_path: &str,
    line: u64,
    character: u64,
    label: &str,
) -> CompletionItem {
    let item = test_bed
        .completion(relative_path, line, character)
        .await
        .unwrap()
        .into_iter()
        .find(|item| item.label == label)
        .unwrap();

    test_bed.client.completion_resolve(item).await.unwrap()
}

async fn run_list(
    test_bed: &TestBed,
    relative_path: &str,
    line: u64,
    character: u64,
) -> Vec<String> {
    test_bed
        .completion(relative_path, line, character)
        .await
        .unwrap()
        .into_iter()
        .map(|item| item.label)
        .sorted()
        .collect()
}

fn verify_text_edit(
    item: &CompletionItem,
    start_line: u64,
    start_character: u64,
    end_line: u64,
    end_character: u64,
    text: &str,
) {
    assert_eq!(
        *item.text_edit.as_ref().unwrap(),
        CompletionTextEdit::Edit(TextEdit::new(
            Range::new_simple(start_line, start_character, end_line, end_character),
            text.into()
        ))
    );
}

fn verify_detail(item: &CompletionItem, detail: &str) {
    assert_eq!(item.detail.as_ref().unwrap(), detail);
}

#[tokio::test]
async fn empty_latex_document() {
    let mut test_bed = TestBedBuilder::new().file("main.tex", "").build().await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_items = test_bed.completion("main.tex", 0, 0).await.unwrap();

    test_bed.shutdown().await;

    assert!(actual_items.is_empty());
}

#[tokio::test]
async fn empty_bibtex_document() {
    let mut test_bed = TestBedBuilder::new().file("main.bib", "").build().await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let actual_items = test_bed.completion("main.bib", 0, 0).await.unwrap();

    test_bed.shutdown().await;

    assert!(actual_items.is_empty());
}

#[tokio::test]
async fn bibtex_comment() {
    let mut test_bed = TestBedBuilder::new().file("main.bib", "foo").build().await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let actual_items = test_bed.completion("main.bib", 0, 2).await.unwrap();

    test_bed.shutdown().await;

    assert!(actual_items.is_empty());
}

#[tokio::test]
async fn bibtex_command_incomplete_entry() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.bib",
            indoc!(
                r#"
                    @article{foo,
                        author = {\LaT
                    }
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let actual_item = run_item(&test_bed, "main.bib", 1, 18, "LaTeX").await;

    test_bed.shutdown().await;

    verify_detail(&actual_item, "built-in");
    verify_text_edit(&actual_item, 1, 15, 1, 18, "LaTeX");
}

#[tokio::test]
async fn bibtex_command_complete_entry() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.bib",
            indoc!(
                r#"
                    @article{foo,
                        author = {\LaT}
                    }
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let actual_item = run_item(&test_bed, "main.bib", 1, 18, "LaTeX").await;

    test_bed.shutdown().await;

    verify_detail(&actual_item, "built-in");
    verify_text_edit(&actual_item, 1, 15, 1, 18, "LaTeX");
}

#[tokio::test]
async fn bibtex_type_empty() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.bib",
            indoc!(
                r#"
                    @
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let actual_item = run_item(&test_bed, "main.bib", 0, 1, "article").await;

    test_bed.shutdown().await;

    assert!(actual_item.documentation.is_some());
    verify_text_edit(&actual_item, 0, 1, 0, 1, "article");
}

#[tokio::test]
async fn bibtex_type_incomplete() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.bib",
            indoc!(
                r#"
                    @art
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let actual_item = run_item(&test_bed, "main.bib", 0, 1, "article").await;

    test_bed.shutdown().await;

    assert!(actual_item.documentation.is_some());
    verify_text_edit(&actual_item, 0, 1, 0, 4, "article");
}

#[tokio::test]
async fn bibtex_type_complete() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.bib",
            indoc!(
                r#"
                    @article
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let actual_item = run_item(&test_bed, "main.bib", 0, 1, "article").await;

    test_bed.shutdown().await;

    assert!(actual_item.documentation.is_some());
    verify_text_edit(&actual_item, 0, 1, 0, 8, "article");
}

#[tokio::test]
async fn bibtex_field_incomplete_entry() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.bib",
            indoc!(
                r#"
                    @article{foo,
                        titl
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let actual_item = run_item(&test_bed, "main.bib", 1, 6, "title").await;

    test_bed.shutdown().await;

    assert!(actual_item.documentation.is_some());
    verify_text_edit(&actual_item, 1, 4, 1, 8, "title");
}

#[tokio::test]
async fn bibtex_field_complete_entry() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.bib",
            indoc!(
                r#"
                    @article{foo,
                        title = {}
                    }
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let actual_item = run_item(&test_bed, "main.bib", 1, 6, "title").await;

    test_bed.shutdown().await;

    assert!(actual_item.documentation.is_some());
    verify_text_edit(&actual_item, 1, 4, 1, 9, "title");
}

#[tokio::test]
async fn latex_begin_command() {
    let mut test_bed = TestBedBuilder::new().file("main.tex", r#"\"#).build().await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_item = run_item(&test_bed, "main.tex", 0, 1, "begin").await;

    test_bed.shutdown().await;

    verify_detail(&actual_item, "built-in");
}

#[cfg(feature = "citation")]
#[tokio::test]
async fn latex_citation_valid() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    \bibliography{main}
                    \begin{document}
                    \cite{
                    \end{document}
                "#
            ),
        )
        .file(
            "main.bib",
            indoc!(
                r#"
                    @article{foo:2019,
                        author = {Foo Bar},
                        title = {Baz Qux},
                        year = {2019},
                    }

                    @article{bar:2005,}
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;
    test_bed.open("main.bib").await;

    let actual_item = run_item(&test_bed, "main.tex", 3, 6, "foo:2019").await;

    test_bed.shutdown().await;

    verify_text_edit(&actual_item, 3, 6, 3, 6, "foo:2019");
    assert_eq!(
        actual_item.documentation.unwrap(),
        Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "Bar, F. (2019). *Baz Qux*.".into()
        })
    );
}

#[cfg(feature = "citation")]
#[tokio::test]
async fn latex_citation_invalid() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    \bibliography{main}
                    \begin{document}
                    \cite{
                    \end{document}
                "#
            ),
        )
        .file(
            "main.bib",
            indoc!(
                r#"
                    @article{foo:2019,
                        author = {Foo Bar},
                        title = {Baz Qux},
                        year = {2019},
                    }

                    @article{bar:2005,}
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;
    test_bed.open("main.bib").await;

    let actual_item = run_item(&test_bed, "main.tex", 3, 6, "bar:2005").await;

    test_bed.shutdown().await;

    verify_text_edit(&actual_item, 3, 6, 3, 6, "bar:2005");
    assert_eq!(actual_item.documentation, None);
}

#[tokio::test]
async fn latex_color_name() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \color{re}
                    \definecolor{foo}{
                    \definecolorset{R}
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_item = run_item(&test_bed, "main.tex", 0, 9, "red").await;

    test_bed.shutdown().await;

    verify_text_edit(&actual_item, 0, 7, 0, 9, "red");
}

#[tokio::test]
async fn latex_color_model_define_color() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \color{re}
                    \definecolor{foo}{
                    \definecolorset{R}
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_item = run_item(&test_bed, "main.tex", 1, 18, "rgb").await;

    test_bed.shutdown().await;

    verify_text_edit(&actual_item, 1, 18, 1, 18, "rgb");
}

#[tokio::test]
async fn latex_model_define_color_set() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \color{re}
                    \definecolor{foo}{
                    \definecolorset{R}
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_item = run_item(&test_bed, "main.tex", 2, 17, "RGB").await;

    test_bed.shutdown().await;

    verify_text_edit(&actual_item, 2, 16, 2, 17, "RGB");
}

#[tokio::test]
async fn latex_component_kernel_command() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{book}
                    \usepackage{amsmath}
                    \chap
                    \varDel
                    \begin{theind}
                    \end{alig}
                    \begin{doc}
                    \vareps                
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_item = run_item(&test_bed, "main.tex", 0, 1, "documentclass").await;

    test_bed.shutdown().await;

    verify_detail(&actual_item, "built-in");
    verify_text_edit(&actual_item, 0, 1, 0, 14, "documentclass");
}

#[tokio::test]
async fn latex_component_kernel_command_glyph() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{book}
                    \usepackage{amsmath}
                    \chap
                    \varDel
                    \begin{theind}
                    \end{alig}
                    \begin{doc}
                    \vareps                
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_item = run_item(&test_bed, "main.tex", 7, 7, "varepsilon").await;

    test_bed.shutdown().await;

    verify_detail(&actual_item, "ε, built-in");
    verify_text_edit(&actual_item, 7, 1, 7, 7, "varepsilon");
}

#[tokio::test]
async fn latex_component_kernel_environment() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{book}
                    \usepackage{amsmath}
                    \chap
                    \varDel
                    \begin{theind}
                    \end{alig}
                    \begin{doc}
                    \vareps                
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_item = run_item(&test_bed, "main.tex", 6, 10, "document").await;

    test_bed.shutdown().await;

    verify_detail(&actual_item, "built-in");
    verify_text_edit(&actual_item, 6, 7, 6, 10, "document");
}

#[tokio::test]
async fn latex_component_class_command() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{book}
                    \usepackage{amsmath}
                    \chap
                    \varDel
                    \begin{theind}
                    \end{alig}
                    \begin{doc}
                    \vareps                
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_item = run_item(&test_bed, "main.tex", 2, 5, "chapter").await;

    test_bed.shutdown().await;

    verify_detail(&actual_item, "book.cls");
    verify_text_edit(&actual_item, 2, 1, 2, 5, "chapter");
}

#[tokio::test]
async fn latex_component_class_environment() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{book}
                    \usepackage{amsmath}
                    \chap
                    \varDel
                    \begin{theind}
                    \end{alig}
                    \begin{doc}
                    \vareps                
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_item = run_item(&test_bed, "main.tex", 4, 13, "theindex").await;

    test_bed.shutdown().await;

    verify_detail(&actual_item, "book.cls");
    verify_text_edit(&actual_item, 4, 7, 4, 13, "theindex");
}

#[tokio::test]
async fn latex_component_package_command() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{book}
                    \usepackage{amsmath}
                    \chap
                    \varDel
                    \begin{theind}
                    \end{alig}
                    \begin{doc}
                    \vareps                
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_item = run_item(&test_bed, "main.tex", 3, 7, "varDelta").await;

    test_bed.shutdown().await;

    verify_detail(&actual_item, "amsmath.sty");
    verify_text_edit(&actual_item, 3, 1, 3, 7, "varDelta");
}

#[tokio::test]
async fn latex_component_package_environment() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{book}
                    \usepackage{amsmath}
                    \chap
                    \varDel
                    \begin{theind}
                    \end{alig}
                    \begin{doc}
                    \vareps                
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_item = run_item(&test_bed, "main.tex", 5, 5, "align").await;

    test_bed.shutdown().await;

    verify_detail(&actual_item, "amsmath.sty");
    verify_text_edit(&actual_item, 5, 5, 5, 9, "align");
}

#[tokio::test]
async fn latex_import_class() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{book}
                    \usepackage{amsmath}            
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_item = run_item(&test_bed, "main.tex", 0, 19, "book").await;

    test_bed.shutdown().await;

    assert!(actual_item.documentation.is_some());
    verify_text_edit(&actual_item, 0, 15, 0, 19, "book");
}

#[tokio::test]
async fn latex_import_package() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{book}
                    \usepackage{amsmath}            
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_item = run_item(&test_bed, "main.tex", 1, 15, "amsmath").await;

    test_bed.shutdown().await;

    assert!(actual_item.documentation.is_some());
    verify_text_edit(&actual_item, 1, 12, 1, 19, "amsmath");
}

#[tokio::test]
async fn latex_include_relative_root_no_extension() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    \include{}
                    \input{}
                    \input{qux/}
                    \addbibresource{}
                "#
            ),
        )
        .file("foo.bib", "")
        .file("bar.tex", "")
        .file("qux/baz.tex", "")
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_items = run_list(&test_bed, "main.tex", 1, 9).await;

    test_bed.shutdown().await;

    assert_eq!(actual_items, vec!["bar", "main", "qux"]);
}

#[tokio::test]
async fn latex_include_relative_root_with_extension() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    \include{}
                    \input{}
                    \input{qux/}
                    \addbibresource{}
                "#
            ),
        )
        .file("foo.bib", "")
        .file("bar.tex", "")
        .file("qux/baz.tex", "")
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_items = run_list(&test_bed, "main.tex", 2, 7).await;

    test_bed.shutdown().await;

    assert_eq!(actual_items, vec!["bar.tex", "main.tex", "qux"]);
}

#[tokio::test]
async fn latex_include_relative_subdir() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    \include{}
                    \input{}
                    \input{qux/}
                    \addbibresource{}
                "#
            ),
        )
        .file("foo.bib", "")
        .file("bar.tex", "")
        .file("qux/baz.tex", "")
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_items = run_list(&test_bed, "main.tex", 3, 11).await;

    test_bed.shutdown().await;

    assert_eq!(actual_items, vec!["baz.tex"]);
}

#[tokio::test]
async fn latex_include_relative_parent_dir() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    \include{}
                    \input{}
                    \input{qux/}
                    \addbibresource{}
                "#
            ),
        )
        .file("foo.bib", "")
        .file("bar.tex", "")
        .file("qux/baz.tex", r#"\input{../}"#)
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("qux/baz.tex").await;

    let actual_items = run_list(&test_bed, "qux/baz.tex", 0, 10).await;

    test_bed.shutdown().await;

    assert_eq!(actual_items, vec!["bar.tex", "main.tex", "qux"]);
}

#[tokio::test]
async fn latex_include_relative_bibliography() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    \include{}
                    \input{}
                    \input{qux/}
                    \addbibresource{}
                "#
            ),
        )
        .file("foo.bib", "")
        .file("bar.tex", "")
        .file("qux/baz.tex", "")
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_items = run_list(&test_bed, "main.tex", 4, 16).await;

    test_bed.shutdown().await;

    assert_eq!(actual_items, vec!["foo.bib", "qux"]);
}

#[tokio::test]
async fn latex_include_root_dir() {
    let mut test_bed = TestBedBuilder::new()
        .file("src/main.tex", r#"\input{}"#)
        .root_dir(".")
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("src/main.tex").await;

    let actual_items = run_list(&test_bed, "src/main.tex", 0, 7).await;

    test_bed.shutdown().await;

    assert_eq!(actual_items, vec!["src"]);
}

#[tokio::test]
async fn latex_label() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "foo.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    %
                    \usepackage{amsmath}
                    \usepackage{caption}
                    \usepackage{amsthm}
                    \newtheorem{lemma}{Lemma}
                    %
                    \begin{document}
                    %
                    \section{Foo}%
                    \label{sec:foo}
                    %
                    \begin{equation}%
                    \label{eq:foo}
                        1 + 1 = 2
                    \end{equation}
                    %
                    \begin{equation}%
                    \label{eq:bar}
                        1 + 1 = 2
                    \end{equation}
                    %
                    \begin{figure}%
                    \LaTeX{}
                    \caption{Baz}%
                    \label{fig:baz}
                    \end{figure}
                    %
                    \begin{lemma}%
                    \label{thm:foo}
                        1 + 1 = 2
                    \end{lemma}
                    %
                    \include{bar}
                    %
                    \end{document}    
                "#
            ),
        )
        .file(
            "foo.aux", 
            indoc!(
                r#"
                    \relax 
                    \@writefile{lof}{\contentsline {figure}{\numberline {1}{\ignorespaces Baz\relax }}{1}\protected@file@percent }
                    \providecommand*\caption@xref[2]{\@setref\relax\@undefined{#1}}
                    \newlabel{fig:baz}{{1}{1}}
                    \@writefile{toc}{\contentsline {section}{\numberline {1}Foo}{1}\protected@file@percent }
                    \newlabel{sec:foo}{{1}{1}}
                    \newlabel{eq:foo}{{1}{1}}
                    \newlabel{eq:bar}{{2}{1}}
                    \newlabel{thm:foo}{{1}{1}}
                    \@input{bar.aux}            
                "#
            ),
        )
        .file(
            "bar.tex", 
            indoc!(
                r#"
                    \section{Bar}%
                    \label{sec:bar}
                    %
                    Lorem ipsum dolor sit amet.
                    \ref{}
                    \eqref{}
                "#
            ),
        )
        .file(
            "bar.aux", 
            indoc!(
                r#"
                    \relax 
                    \@writefile{toc}{\contentsline {section}{\numberline {2}Bar}{2}\protected@file@percent }
                    \newlabel{sec:bar}{{2}{2}}
                    \@setckpt{bar}{
                    \setcounter{page}{3}
                    \setcounter{equation}{2}
                    \setcounter{enumi}{0}
                    \setcounter{enumii}{0}
                    \setcounter{enumiii}{0}
                    \setcounter{enumiv}{0}
                    \setcounter{footnote}{0}
                    \setcounter{mpfootnote}{0}
                    \setcounter{part}{0}
                    \setcounter{section}{2}
                    \setcounter{subsection}{0}
                    \setcounter{subsubsection}{0}
                    \setcounter{paragraph}{0}
                    \setcounter{subparagraph}{0}
                    \setcounter{figure}{1}
                    \setcounter{table}{0}
                    \setcounter{parentequation}{0}
                    \setcounter{caption@flags}{0}
                    \setcounter{ContinuedFloat}{0}
                    \setcounter{lemma}{1}
                    }
                "#),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("foo.tex").await;
    test_bed.open("foo.aux").await;
    test_bed.open("bar.tex").await;
    test_bed.open("bar.aux").await;

    let actual_items = test_bed.completion("bar.tex", 4, 5).await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_items.len(), 6);
    verify_text_edit(&actual_items[0], 4, 5, 4, 5, "sec:bar");
    verify_text_edit(&actual_items[1], 4, 5, 4, 5, "sec:foo");
    verify_text_edit(&actual_items[2], 4, 5, 4, 5, "eq:foo");
    verify_text_edit(&actual_items[3], 4, 5, 4, 5, "eq:bar");
    verify_text_edit(&actual_items[4], 4, 5, 4, 5, "fig:baz");
    verify_text_edit(&actual_items[5], 4, 5, 4, 5, "thm:foo");
    verify_detail(&actual_items[0], "Section 2 (Bar)");
    verify_detail(&actual_items[1], "Section 1 (Foo)");
    verify_detail(&actual_items[2], "Equation (1)");
    verify_detail(&actual_items[3], "Equation (2)");
    verify_detail(&actual_items[4], "Figure 1");
    verify_detail(&actual_items[5], "Lemma 1");
    assert_eq!(
        *actual_items[4].documentation.as_ref().unwrap(),
        Documentation::String("Baz".into())
    );
}

#[tokio::test]
async fn latex_preselect_environment() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \begin{document}
                    \end{                          
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_item = run_item(&test_bed, "main.tex", 1, 5, "document").await;

    test_bed.shutdown().await;

    assert!(actual_item.preselect.unwrap());
}

#[tokio::test]
async fn latex_theorem() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    \usepackage{amsthm}
                    \newtheorem{foo}{Foo}
                    \begin{f}                        
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_item = run_item(&test_bed, "main.tex", 3, 7, "foo").await;

    test_bed.shutdown().await;

    verify_text_edit(&actual_item, 3, 7, 3, 8, "foo");
    verify_detail(&actual_item, "user-defined");
}

#[tokio::test]
async fn latex_pgf_library() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \usepackage{tikz}
                    \usepgflibrary{}
                    \usetikzlibrary{}                    
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_item = run_item(&test_bed, "main.tex", 1, 15, "arrows").await;

    test_bed.shutdown().await;

    verify_text_edit(&actual_item, 1, 15, 1, 15, "arrows");
}

#[tokio::test]
async fn latex_tikz_library() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \usepackage{tikz}
                    \usepgflibrary{}
                    \usetikzlibrary{}                    
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_item = run_item(&test_bed, "main.tex", 2, 16, "arrows").await;

    test_bed.shutdown().await;

    verify_text_edit(&actual_item, 2, 16, 2, 16, "arrows");
}

#[tokio::test]
async fn latex_user_command() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \foo
                    \fo
                    \begin{foo}
                    \end{foo}
                    \begin{fo}     
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_item = run_item(&test_bed, "main.tex", 1, 3, "foo").await;

    test_bed.shutdown().await;

    verify_detail(&actual_item, "user-defined");
    verify_text_edit(&actual_item, 1, 1, 1, 3, "foo");
}

#[tokio::test]
async fn latex_user_environment() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \foo
                    \fo
                    \begin{foo}
                    \end{foo}
                    \begin{fo}     
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_item = run_item(&test_bed, "main.tex", 4, 7, "foo").await;

    test_bed.shutdown().await;

    verify_detail(&actual_item, "user-defined");
    verify_text_edit(&actual_item, 4, 7, 4, 9, "foo");
}
