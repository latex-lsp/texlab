use texlab_test::completion::*;

const SCENARIO: &str = "bibtex/command";

#[tokio::test]
async fn incomplete_entry() {
    let item = run_item(SCENARIO, "foo.bib", 1, 15, "LaTeX").await;
    verify::text_edit(&item, 1, 15, 1, 18, "LaTeX");
    verify::detail(&item, "built-in");
}

#[tokio::test]
async fn complete_entry() {
    let item = run_item(SCENARIO, "foo.bib", 5, 15, "LaTeX").await;
    verify::text_edit(&item, 5, 15, 5, 18, "LaTeX");
    verify::detail(&item, "built-in");
}
