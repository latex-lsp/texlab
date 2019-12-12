use texlab_protocol::{Range, RangeExt};
use texlab_test::definition::*;

const SCENARIO: &str = "latex/label";

#[tokio::test]
async fn default_link() {
    let (scenario, mut links) = run_link(SCENARIO, "default.tex", 1, 7).await;
    assert_eq!(links.len(), 1);
    let link = links.pop().unwrap();
    verify::origin_selection_range(&link, 1, 5, 1, 8);
    assert_eq!(link.target_uri, scenario.uri("default.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 0, 11));
    assert_eq!(link.target_selection_range, Range::new_simple(0, 0, 0, 11));
}

#[tokio::test]
async fn equation_link() {
    let (scenario, mut links) = run_link(SCENARIO, "equation.tex", 5, 8).await;
    assert_eq!(links.len(), 1);
    let link = links.pop().unwrap();
    verify::origin_selection_range(&link, 5, 5, 5, 11);
    assert_eq!(link.target_uri, scenario.uri("equation.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 3, 14));
    assert_eq!(link.target_selection_range, Range::new_simple(1, 0, 1, 14));
}

#[tokio::test]
async fn float_link() {
    let (scenario, mut links) = run_link(SCENARIO, "float.tex", 6, 6).await;
    assert_eq!(links.len(), 1);
    let link = links.pop().unwrap();
    verify::origin_selection_range(&link, 6, 5, 6, 8);
    assert_eq!(link.target_uri, scenario.uri("float.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 4, 12));
    assert_eq!(link.target_selection_range, Range::new_simple(3, 0, 3, 11));
}

#[tokio::test]
async fn item_link() {
    let (scenario, mut links) = run_link(SCENARIO, "item.tex", 6, 6).await;
    assert_eq!(links.len(), 1);
    let link = links.pop().unwrap();
    verify::origin_selection_range(&link, 6, 5, 6, 8);
    assert_eq!(link.target_uri, scenario.uri("item.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 4, 15));
    assert_eq!(link.target_selection_range, Range::new_simple(2, 9, 2, 20));
}

#[tokio::test]
async fn section_link() {
    let (scenario, mut links) = run_link(SCENARIO, "section.tex", 6, 6).await;
    assert_eq!(links.len(), 1);
    let link = links.pop().unwrap();
    verify::origin_selection_range(&link, 6, 5, 6, 12);
    assert_eq!(link.target_uri, scenario.uri("section.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 3, 0));
    assert_eq!(link.target_selection_range, Range::new_simple(1, 0, 1, 15));
}

#[tokio::test]
async fn theorem_link() {
    let (scenario, mut links) = run_link(SCENARIO, "theorem.tex", 8, 7).await;
    assert_eq!(links.len(), 1);
    let link = links.pop().unwrap();
    verify::origin_selection_range(&link, 8, 5, 8, 12);
    assert_eq!(link.target_uri, scenario.uri("theorem.tex").into());
    assert_eq!(link.target_range, Range::new_simple(3, 0, 6, 11));
    assert_eq!(link.target_selection_range, Range::new_simple(4, 0, 4, 15));
}
