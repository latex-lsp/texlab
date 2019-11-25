pub mod support;

use lsp_types::Range;
use support::symbol::*;
use texlab::range::RangeExt;

#[tokio::test]
async fn enumerate() {
    let mut symbols = run_hierarchical("enumerate.tex").await;
    assert_eq!(symbols.len(), 1);
    verify::symbol(
        &symbols[0],
        "Enumerate",
        None,
        Range::new_simple(4, 0, 9, 15),
        Range::new_simple(4, 0, 9, 15),
    );

    let children = symbols[0].children.take().unwrap();
    assert_eq!(children.len(), 4);
    verify::symbol(
        &children[0],
        "1",
        Some("it:foo"),
        Range::new_simple(5, 9, 5, 23),
        Range::new_simple(5, 4, 6, 4),
    );
    verify::symbol(
        &children[1],
        "Item",
        Some("it:bar"),
        Range::new_simple(6, 9, 6, 23),
        Range::new_simple(6, 4, 7, 4),
    );
    verify::symbol(
        &children[2],
        "Baz",
        None,
        Range::new_simple(7, 4, 7, 14),
        Range::new_simple(7, 4, 8, 4),
    );
    verify::symbol(
        &children[3],
        "Qux",
        Some("it:qux"),
        Range::new_simple(8, 14, 8, 28),
        Range::new_simple(8, 4, 9, 0),
    );
}

#[tokio::test]
async fn equation() {
    let symbols = run_hierarchical("equation.tex").await;
    assert_eq!(symbols.len(), 3);
    verify::symbol(
        &symbols[0],
        "Equation (1)",
        Some("eq:foo"),
        Range::new_simple(4, 16, 4, 30),
        Range::new_simple(4, 0, 6, 14),
    );
    verify::symbol(
        &symbols[1],
        "Equation",
        Some("eq:bar"),
        Range::new_simple(8, 16, 8, 30),
        Range::new_simple(8, 0, 10, 14),
    );
    verify::symbol(
        &symbols[2],
        "Equation",
        None,
        Range::new_simple(12, 0, 14, 14),
        Range::new_simple(12, 0, 14, 14),
    );
}

#[tokio::test]
async fn float() {
    let symbols = run_hierarchical("float.tex").await;
    assert_eq!(symbols.len(), 3);
    verify::symbol(
        &symbols[0],
        "Figure 1: Foo",
        Some("fig:foo"),
        Range::new_simple(6, 17, 6, 32),
        Range::new_simple(4, 0, 7, 12),
    );
    verify::symbol(
        &symbols[1],
        "Figure: Bar",
        Some("fig:bar"),
        Range::new_simple(11, 17, 11, 32),
        Range::new_simple(9, 0, 12, 12),
    );
    verify::symbol(
        &symbols[2],
        "Figure: Baz",
        None,
        Range::new_simple(14, 0, 17, 12),
        Range::new_simple(14, 0, 17, 12),
    );
}

#[tokio::test]
async fn section() {
    let mut symbols = run_hierarchical("section.tex").await;
    assert_eq!(symbols.len(), 2);
    verify::symbol(
        &symbols[0],
        "Foo",
        None,
        Range::new_simple(4, 0, 4, 13),
        Range::new_simple(4, 0, 6, 0),
    );
    verify::symbol(
        &symbols[1],
        "2 Bar",
        Some("sec:bar"),
        Range::new_simple(6, 0, 6, 13),
        Range::new_simple(6, 0, 10, 0),
    );

    let children = symbols[1].children.take().unwrap();
    assert_eq!(children.len(), 1);
    verify::symbol(
        &children[0],
        "Baz",
        Some("sec:baz"),
        Range::new_simple(8, 0, 8, 16),
        Range::new_simple(8, 0, 10, 0),
    );
}

#[tokio::test]
async fn theorem() {
    let symbols = run_hierarchical("theorem.tex").await;
    assert_eq!(symbols.len(), 4);
    verify::symbol(
        &symbols[0],
        "Lemma 1 (Foo)",
        Some("thm:foo"),
        Range::new_simple(6, 18, 6, 33),
        Range::new_simple(6, 0, 8, 11),
    );
    verify::symbol(
        &symbols[1],
        "Lemma 2",
        Some("thm:bar"),
        Range::new_simple(10, 13, 10, 28),
        Range::new_simple(10, 0, 12, 11),
    );
    verify::symbol(
        &symbols[2],
        "Lemma",
        Some("thm:baz"),
        Range::new_simple(14, 13, 14, 28),
        Range::new_simple(14, 0, 16, 11),
    );
    verify::symbol(
        &symbols[3],
        "Lemma (Qux)",
        None,
        Range::new_simple(18, 0, 20, 11),
        Range::new_simple(18, 0, 20, 11),
    );
}
