#![feature(async_await)]

use futures::executor::block_on;
use itertools::Itertools;
use lsp_types::*;
use texlab::scenario::Scenario;

pub async fn run(scenario: &'static str, file: &'static str, position: Position) -> Vec<String> {
    let scenario = format!("completion/{}", scenario);
    let scenario = Scenario::new(&scenario).await;
    scenario.open(file).await;

    let params = CompletionParams {
        text_document: TextDocumentIdentifier::new(scenario.uri(file)),
        position,
        context: None,
    };
    scenario
        .server
        .completion(params)
        .await
        .unwrap()
        .items
        .into_iter()
        .map(|item| item.label.into_owned())
        .sorted()
        .collect()
}

#[test]
fn test_kernel_command() {
    block_on(async move {
        let items = run("kernel", "foo.tex", Position::new(2, 5)).await;
        assert_eq!(items.iter().any(|item| item == "usepackage"), true);
    });
}

#[test]
fn test_kernel_command_bibtex() {
    block_on(async move {
        let items = run("kernel", "foo.bib", Position::new(1, 17)).await;
        assert_eq!(items.iter().any(|item| item == "LaTeX"), true);
    });
}

#[test]
fn test_kernel_environment() {
    block_on(async move {
        let items = run("kernel", "foo.tex", Position::new(4, 10)).await;
        assert_eq!(items.iter().any(|item| item == "document"), true);
    });
}

#[test]
fn test_user_command() {
    block_on(async move {
        let items = run("user", "foo.tex", Position::new(2, 3)).await;
        assert_eq!(items.iter().all(|item| item != "fo"), true);
        assert_eq!(items.iter().any(|item| item == "foo"), true);
    });
}

#[test]
fn test_label() {
    block_on(async move {
        let items = run("label", "foo.tex", Position::new(5, 5)).await;
        assert_eq!(items, vec!["bar", "baz", "foo"]);
    });
}

#[test]
fn test_citation() {
    block_on(async move {
        let items = run("citation", "foo.tex", Position::new(3, 6)).await;
        assert_eq!(items, vec!["bar", "baz", "foo"]);
    });
}

#[test]
fn test_symbol_command_kernel() {
    block_on(async move {
        let items = run("symbol", "foo.tex", Position::new(0, 1)).await;
        assert_eq!(items.iter().any(|item| item == "varepsilon"), true);
    });
}

#[test]
fn test_symbol_argument() {
    block_on(async move {
        let items = run("symbol", "foo.tex", Position::new(1, 8)).await;
        assert_eq!(items.len(), 26);
        assert_eq!(items[0], "A");
    });
}

#[test]
fn test_color() {
    block_on(async move {
        let items = run("color", "foo.tex", Position::new(0, 10)).await;
        assert_eq!(items.iter().any(|item| item == "black"), true);
    });
}

#[test]
fn test_color_model() {
    block_on(async move {
        let items = run("color", "foo.tex", Position::new(1, 18)).await;
        assert_eq!(items.iter().any(|item| item == "rgb"), true);
    });
}

#[test]
fn test_include_top_level() {
    block_on(async move {
        let items = run("include", "foo.tex", Position::new(0, 9)).await;
        assert_eq!(items, vec!["bar", "foo", "qux"]);
    });
}

#[test]
fn test_include_directory() {
    block_on(async move {
        let items = run("include", "foo.tex", Position::new(1, 11)).await;
        assert_eq!(items, vec!["bar.tex", "baz.tex"]);
    });
}

#[test]
fn test_include_bibliography() {
    block_on(async move {
        let items = run("include", "bar/baz.tex", Position::new(0, 16)).await;
        assert_eq!(items, vec!["foo.bib"]);
    });
}

#[test]
fn test_include_graphics() {
    block_on(async move {
        let items = run("include", "bar/baz.tex", Position::new(1, 17)).await;
        assert_eq!(items, vec!["image1.png", "image2.jpg"]);
    });
}

#[test]
fn test_include_graphics_svg() {
    block_on(async move {
        let items = run("include", "bar/baz.tex", Position::new(2, 12)).await;
        assert_eq!(items, vec!["image3"]);
    });
}

#[test]
fn test_pgf_library() {
    block_on(async move {
        let items = run("pgf_library", "foo.tex", Position::new(0, 18)).await;
        assert_eq!(items.iter().any(|item| item == "arrows"), true);
    });
}

#[test]
fn test_tikz_library() {
    block_on(async move {
        let items = run("tikz_library", "foo.tex", Position::new(0, 19)).await;
        assert_eq!(items.iter().any(|item| item == "arrows"), true);
    });
}

#[test]
fn test_entry_type() {
    block_on(async move {
        let items = run("entry_type", "foo.bib", Position::new(0, 1)).await;
        assert_eq!(items.iter().any(|item| item == "article"), true);
    });
}

#[test]
fn test_entry_type_preamble() {
    block_on(async move {
        let items = run("entry_type", "foo.bib", Position::new(1, 3)).await;
        assert_eq!(items.iter().any(|item| item == "preamble"), true);
    });
}

#[test]
fn test_entry_type_string() {
    block_on(async move {
        let items = run("entry_type", "foo.bib", Position::new(2, 3)).await;
        assert_eq!(items.iter().any(|item| item == "string"), true);
    });
}

#[test]
fn test_entry_type_comment() {
    block_on(async move {
        let items = run("entry_type", "foo.bib", Position::new(3, 3)).await;
        assert_eq!(items.iter().any(|item| item == "comment"), true);
    });
}

#[test]
fn test_field_name() {
    block_on(async move {
        let items = run("field_name", "foo.bib", Position::new(1, 7)).await;
        assert_eq!(items.iter().any(|item| item == "author"), true);
    });
}
