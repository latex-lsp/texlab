pub mod support;

use support::symbol::*;

#[tokio::test]
async fn filter_type_section() {
    let (scenario, symbols) = run_workspace("section").await;
    assert_eq!(symbols.len(), 4);
    verify::symbol_info(&symbols[0], &scenario, "foo.tex", "1 Foo", 07, 0, 13, 0);
    verify::symbol_info(&symbols[1], &scenario, "foo.tex", "2 Bar", 13, 0, 21, 0);
    verify::symbol_info(&symbols[2], &scenario, "foo.tex", "3 Baz", 21, 0, 29, 0);
    verify::symbol_info(&symbols[3], &scenario, "foo.tex", "4 Qux", 29, 0, 37, 0);
}

#[tokio::test]
async fn filter_type_figure() {
    let (scenario, symbols) = run_workspace("figure").await;
    assert_eq!(symbols.len(), 1);
    let name = "Figure 1: Bar";
    verify::symbol_info(&symbols[0], &scenario, "foo.tex", name, 15, 0, 19, 12);
}

#[tokio::test]
async fn filter_type_item() {
    let (scenario, symbols) = run_workspace("item").await;
    assert_eq!(symbols.len(), 3);
    verify::symbol_info(&symbols[0], &scenario, "foo.tex", "1", 24, 4, 25, 4);
    verify::symbol_info(&symbols[1], &scenario, "foo.tex", "2", 25, 4, 26, 4);
    verify::symbol_info(&symbols[2], &scenario, "foo.tex", "3", 26, 4, 27, 0);
}

#[tokio::test]
async fn filter_type_math() {
    let (scenario, symbols) = run_workspace("math").await;
    assert_eq!(symbols.len(), 2);
    let name1 = "Equation (1)";
    let name2 = "Lemma 1 (Qux)";
    verify::symbol_info(&symbols[0], &scenario, "foo.tex", name1, 9, 0, 11, 14);
    verify::symbol_info(&symbols[1], &scenario, "foo.tex", name2, 33, 0, 35, 11);
}

#[tokio::test]
async fn filter_bibtex() {
    let (scenario, symbols) = run_workspace("bibtex").await;
    assert_eq!(symbols.len(), 2);
    verify::symbol_info(&symbols[0], &scenario, "bar.bib", "foo", 0, 0, 0, 14);
    verify::symbol_info(&symbols[1], &scenario, "bar.bib", "bar", 2, 0, 2, 20);
}
