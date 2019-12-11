pub mod support;

use support::completion::*;
use texlab_protocol::Documentation;

const SCENARIO: &str = "latex/label";

#[tokio::test]
async fn default() {
    let (_, items) = run_list(SCENARIO, "bar.tex", 4, 5).await;
    assert_eq!(items.len(), 6);
    verify::text_edit(&items[0], 4, 5, 4, 5, "sec:bar");
    verify::text_edit(&items[1], 4, 5, 4, 5, "sec:foo");
    verify::text_edit(&items[2], 4, 5, 4, 5, "eq:foo");
    verify::text_edit(&items[3], 4, 5, 4, 5, "eq:bar");
    verify::text_edit(&items[4], 4, 5, 4, 5, "fig:baz");
    verify::text_edit(&items[5], 4, 5, 4, 5, "thm:foo");
    verify::detail(&items[0], "Section 2 (Bar)");
    verify::detail(&items[1], "Section 1 (Foo)");
    verify::detail(&items[2], "Equation (1)");
    verify::detail(&items[3], "Equation (2)");
    verify::detail(&items[4], "Figure 1");
    verify::detail(&items[5], "Lemma 1");
    assert_eq!(
        *items[4].documentation.as_ref().unwrap(),
        Documentation::String("Baz".into())
    );
}

#[tokio::test]
async fn equation() {
    let (_, items) = run_list(SCENARIO, "bar.tex", 5, 7).await;
    verify::labels(&items, vec!["eq:bar", "eq:foo"]);
}
