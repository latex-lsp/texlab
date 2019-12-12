use texlab_test::completion::*;

const SCENARIO: &str = "bibtex/type";

#[tokio::test]
async fn empty_type() {
    let item = run_item(SCENARIO, "foo.bib", 0, 1, "article").await;
    assert!(item.documentation.is_some());
    verify::text_edit(&item, 0, 1, 0, 1, "article");
}

#[tokio::test]
async fn incomplete_type() {
    let item = run_item(SCENARIO, "foo.bib", 1, 2, "article").await;
    assert!(item.documentation.is_some());
    verify::text_edit(&item, 1, 1, 1, 4, "article");
}

#[tokio::test]
async fn complete_type() {
    let item = run_item(SCENARIO, "foo.bib", 2, 8, "article").await;
    assert!(item.documentation.is_some());
    verify::text_edit(&item, 2, 1, 2, 8, "article");
}
