#![feature(async_await)]

use itertools::Itertools;
use lsp_types::*;
use texlab::scenario::Scenario;

pub async fn run(scenario: &'static str, file: &'static str, position: Position) -> Vec<String> {
    let scenario = format!("completion/{}", scenario);
    let scenario = Scenario::new(&scenario).await;
    scenario.open(file).await;
    scenario.server.stop_scanning().await;

    let params = CompletionParams {
        text_document: TextDocumentIdentifier::new(scenario.uri(file)),
        position,
        context: None,
    };

    let items = scenario
        .server
        .completion(params)
        .await
        .unwrap()
        .items
        .into_iter()
        .map(|item| (*item.label).to_owned())
        .sorted()
        .collect();

    scenario.directory.close().unwrap();
    items
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_kernel_command() {
    let items = run("kernel", "foo.tex", Position::new(2, 5)).await;
    assert!(items.iter().any(|item| item == "usepackage"));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_kernel_command_bibtex() {
    let items = run("kernel", "foo.bib", Position::new(1, 17)).await;
    assert!(items.iter().any(|item| item == "LaTeX"));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_kernel_environment() {
    let items = run("kernel", "foo.tex", Position::new(4, 10)).await;
    assert!(items.iter().any(|item| item == "document"));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_user_command() {
    let items = run("user", "foo.tex", Position::new(2, 3)).await;
    assert!(items.iter().all(|item| item != "fo"));
    assert!(items.iter().any(|item| item == "foo"));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_label() {
    let items = run("label", "foo.tex", Position::new(5, 5)).await;
    assert_eq!(items, vec!["bar", "baz", "foo"]);
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_citation() {
    let items = run("citation", "foo.tex", Position::new(3, 6)).await;
    assert_eq!(items, vec!["bar", "baz", "foo"]);
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_symbol_command_kernel() {
    let items = run("symbol", "foo.tex", Position::new(0, 1)).await;
    assert!(items.iter().any(|item| item == "varepsilon"));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_symbol_argument() {
    let items = run("symbol", "foo.tex", Position::new(1, 8)).await;
    assert_eq!(items.len(), 26);
    assert_eq!(items[0], "A");
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_color() {
    let items = run("color", "foo.tex", Position::new(0, 10)).await;
    assert!(items.iter().any(|item| item == "black"));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_color_model() {
    let items = run("color", "foo.tex", Position::new(1, 18)).await;
    assert!(items.iter().any(|item| item == "rgb"));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_include_top_level() {
    let items = run("include", "foo.tex", Position::new(0, 9)).await;
    assert_eq!(items, vec!["bar", "foo", "qux"]);
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_include_directory() {
    let items = run("include", "foo.tex", Position::new(1, 11)).await;
    assert_eq!(items, vec!["bar.tex", "baz.tex"]);
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_include_bibliography() {
    let items = run("include", "bar/baz.tex", Position::new(0, 16)).await;
    assert_eq!(items, vec!["foo.bib"]);
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_include_graphics() {
    let items = run("include", "bar/baz.tex", Position::new(1, 17)).await;
    assert_eq!(items, vec!["image1.png", "image2.jpg"]);
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_include_graphics_svg() {
    let items = run("include", "bar/baz.tex", Position::new(2, 12)).await;
    assert_eq!(items, vec!["image3"]);
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_import_class() {
    let items = run("import", "foo.tex", Position::new(0, 18)).await;
    assert!(items.iter().any(|item| item == "article"));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_import_package() {
    let items = run("import", "foo.tex", Position::new(1, 15)).await;
    assert!(items.iter().any(|item| item == "amsmath"));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_package_command() {
    let items = run("component", "foo.tex", Position::new(2, 3)).await;
    assert!(items.iter().any(|item| item == "AmS"));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_package_environment() {
    let items = run("component", "foo.tex", Position::new(3, 11)).await;
    assert!(items.iter().any(|item| item == "align"));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_class_command() {
    let items = run("component", "foo.tex", Position::new(4, 8)).await;
    assert!(items.iter().any(|item| item == "thetable"));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_class_environment() {
    let items = run("component", "foo.tex", Position::new(5, 14)).await;
    assert!(items.iter().any(|item| item == "theindex"));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_pgf_library() {
    let items = run("pgf_library", "foo.tex", Position::new(0, 18)).await;
    assert!(items.iter().any(|item| item == "arrows"));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_tikz_library() {
    let items = run("tikz_library", "foo.tex", Position::new(0, 19)).await;
    assert!(items.iter().any(|item| item == "arrows"));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_entry_type() {
    let items = run("entry_type", "foo.bib", Position::new(0, 1)).await;
    assert!(items.iter().any(|item| item == "article"));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_entry_type_preamble() {
    let items = run("entry_type", "foo.bib", Position::new(1, 3)).await;
    assert!(items.iter().any(|item| item == "preamble"));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_entry_type_string() {
    let items = run("entry_type", "foo.bib", Position::new(2, 3)).await;
    assert!(items.iter().any(|item| item == "string"));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_entry_type_comment() {
    let items = run("entry_type", "foo.bib", Position::new(3, 3)).await;
    assert!(items.iter().any(|item| item == "comment"));
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_field_name() {
    let items = run("field_name", "foo.bib", Position::new(1, 7)).await;
    assert!(items.iter().any(|item| item == "author"));
}
