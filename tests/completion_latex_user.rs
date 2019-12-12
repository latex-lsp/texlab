use texlab_test::completion::*;

const SCENARIO: &str = "latex/user";

#[tokio::test]
async fn command() {
    let item = run_item(SCENARIO, "foo.tex", 1, 3, "foo").await;
    verify::text_edit(&item, 1, 1, 1, 3, "foo");
    verify::detail(&item, "user-defined");
}

#[tokio::test]
async fn environment() {
    let item = run_item(SCENARIO, "foo.tex", 4, 7, "foo").await;
    verify::text_edit(&item, 4, 7, 4, 9, "foo");
    verify::detail(&item, "user-defined");
}
