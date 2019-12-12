use texlab_test::completion::*;

const SCENARIO: &str = "bibtex/field";

#[tokio::test]
async fn incomplete_entry() {
    let item = run_item(SCENARIO, "foo.bib", 1, 6, "title").await;
    assert!(item.documentation.is_some());
    verify::text_edit(&item, 1, 4, 1, 8, "title");
}

#[tokio::test]
async fn complete_entry() {
    let item = run_item(SCENARIO, "foo.bib", 4, 5, "title").await;
    assert!(item.documentation.is_some());
    verify::text_edit(&item, 4, 4, 4, 9, "title");
}
