pub mod support;

use support::completion::*;

const SCENARIO: &str = "latex/color";

#[tokio::test]
async fn name() {
    let item = run_item(SCENARIO, "foo.tex", 0, 9, "red").await;
    verify::text_edit(&item, 0, 7, 0, 9, "red");
}

#[tokio::test]
async fn model_definecolor() {
    let item = run_item(SCENARIO, "foo.tex", 1, 18, "rgb").await;
    verify::text_edit(&item, 1, 18, 1, 18, "rgb");
}

#[tokio::test]
async fn model_definecolorset() {
    let item = run_item(SCENARIO, "foo.tex", 2, 17, "RGB").await;
    verify::text_edit(&item, 2, 16, 2, 17, "RGB");
}
