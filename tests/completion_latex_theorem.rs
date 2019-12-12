use texlab_test::completion::*;

const SCENARIO: &str = "latex/theorem";

#[tokio::test]
async fn newtheorem() {
    let item = run_item(SCENARIO, "foo.tex", 4, 7, "foo").await;
    verify::text_edit(&item, 4, 7, 4, 8, "foo");
    verify::detail(&item, "user-defined");
}
