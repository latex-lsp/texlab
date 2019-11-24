pub mod support;

use support::completion::*;

const SCENARIO: &str = "latex/component";

#[tokio::test]
async fn kernel_command() {
    let item = run_item(SCENARIO, "foo.tex", 0, 1, "documentclass").await;
    verify::text_edit(&item, 0, 1, 0, 14, "documentclass");
    verify::detail(&item, "built-in");
}

#[tokio::test]
async fn kernel_command_glyph() {
    let item = run_item(SCENARIO, "foo.tex", 7, 7, "varepsilon").await;
    verify::text_edit(&item, 7, 1, 7, 7, "varepsilon");
    verify::detail(&item, "ε, built-in");
}

#[tokio::test]
async fn kernel_environment() {
    let item = run_item(SCENARIO, "foo.tex", 6, 10, "document").await;
    verify::text_edit(&item, 6, 7, 6, 10, "document");
    verify::detail(&item, "built-in");
}

#[tokio::test]
async fn class_import() {
    let item = run_item(SCENARIO, "foo.tex", 0, 19, "book").await;
    verify::text_edit(&item, 0, 15, 0, 19, "book");
    assert!(item.documentation.is_some());
}

#[tokio::test]
async fn class_command() {
    let item = run_item(SCENARIO, "foo.tex", 2, 5, "chapter").await;
    verify::text_edit(&item, 2, 1, 2, 5, "chapter");
    verify::detail(&item, "book.cls");
}

#[tokio::test]
async fn class_environment() {
    let item = run_item(SCENARIO, "foo.tex", 4, 13, "theindex").await;
    verify::text_edit(&item, 4, 7, 4, 13, "theindex");
    verify::detail(&item, "book.cls");
}

#[tokio::test]
async fn package_import() {
    let item = run_item(SCENARIO, "foo.tex", 1, 15, "amsmath").await;
    verify::text_edit(&item, 1, 12, 1, 19, "amsmath");
    assert!(item.documentation.is_some());
}

#[tokio::test]
async fn package_command() {
    let item = run_item(SCENARIO, "foo.tex", 3, 7, "varDelta").await;
    verify::text_edit(&item, 3, 1, 3, 7, "varDelta");
    verify::detail(&item, "amsmath.sty");
}

#[tokio::test]
async fn package_environment() {
    let item = run_item(SCENARIO, "foo.tex", 5, 5, "align").await;
    verify::text_edit(&item, 5, 5, 5, 9, "align");
    verify::detail(&item, "amsmath.sty");
}
