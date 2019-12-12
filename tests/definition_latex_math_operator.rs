use texlab_protocol::{Range, RangeExt};
use texlab_test::definition::*;

const SCENARIO: &str = "latex/math_operator";

#[tokio::test]
async fn link() {
    let (scenario, mut links) = run_link(SCENARIO, "foo.tex", 2, 2).await;
    assert_eq!(links.len(), 1);
    let link = links.pop().unwrap();
    verify::origin_selection_range(&link, 2, 0, 2, 4);
    assert_eq!(link.target_uri, scenario.uri("foo.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 0, 31));
    assert_eq!(link.target_selection_range, Range::new_simple(0, 0, 0, 31));
}
