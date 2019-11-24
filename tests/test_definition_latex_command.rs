pub mod support;

use lsp_types::Range;
use support::definition::*;
use texlab::range::RangeExt;

const SCENARIO: &str = "latex/command";

#[tokio::test]
async fn link() {
    let (scenario, mut links) = run_link(SCENARIO, "foo.tex", 2, 2).await;
    assert_eq!(links.len(), 1);
    let link = links.pop().unwrap();
    verify::origin_selection_range(&link, 2, 0, 2, 4);
    assert_eq!(link.target_uri, scenario.uri("foo.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 0, 22));
    assert_eq!(link.target_selection_range, Range::new_simple(0, 0, 0, 22));
}
