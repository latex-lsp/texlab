pub mod support;

use lsp_types::Range;
use support::definition::*;
use texlab_protocol::RangeExt;

const SCENARIO: &str = "bibtex/string";

#[tokio::test]
async fn link() {
    let (scenario, mut links) = run_link(SCENARIO, "foo.bib", 5, 14).await;
    assert_eq!(links.len(), 1);
    let link = links.pop().unwrap();
    verify::origin_selection_range(&link, 5, 13, 5, 16);
    assert_eq!(link.target_uri, scenario.uri("foo.bib").into());
    assert_eq!(link.target_range, Range::new_simple(2, 0, 2, 20));
    assert_eq!(link.target_selection_range, Range::new_simple(2, 8, 2, 11));
}
