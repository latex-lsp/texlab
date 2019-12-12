use texlab_test::completion::*;

const SCENARIO: &str = "latex/tikz";

#[tokio::test]
async fn pgf_library() {
    let item = run_item(SCENARIO, "foo.tex", 1, 15, "arrows").await;
    verify::text_edit(&item, 1, 15, 1, 15, "arrows");
}

#[tokio::test]
async fn tikz_library() {
    let item = run_item(SCENARIO, "foo.tex", 2, 16, "arrows").await;
    verify::text_edit(&item, 2, 16, 2, 16, "arrows");
}
