use texlab_protocol::{Range, RangeExt};
use texlab_test::definition::*;

const SCENARIO: &str = "latex/citation";

#[tokio::test]
async fn link() {
    let (scenario, mut links) = run_link(SCENARIO, "foo.tex", 1, 7).await;
    assert_eq!(links.len(), 1);
    let link = links.pop().unwrap();
    verify::origin_selection_range(&link, 1, 6, 1, 9);
    assert_eq!(link.target_uri, scenario.uri("bar.bib").into());
    assert_eq!(link.target_range, Range::new_simple(2, 0, 2, 14));
    assert_eq!(link.target_selection_range, Range::new_simple(2, 9, 2, 12));
}

#[tokio::test]
async fn location() {
    let (scenario, mut locations) = run_location(SCENARIO, "foo.tex", 1, 7).await;
    assert_eq!(locations.len(), 1);
    let location = locations.pop().unwrap();
    assert_eq!(location.uri, scenario.uri("bar.bib").into());
    assert_eq!(location.range, Range::new_simple(2, 9, 2, 12));
}
