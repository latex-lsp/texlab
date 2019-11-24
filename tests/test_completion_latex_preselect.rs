pub mod support;

use support::completion::*;

const SCENARIO: &str = "latex/preselect";

#[tokio::test]
async fn environment() {
    let item = run_item(SCENARIO, "foo.tex", 2, 5, "document").await;
    assert_eq!(item.preselect, Some(true));
}
