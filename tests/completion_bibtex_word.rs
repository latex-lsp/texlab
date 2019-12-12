use texlab_test::completion::*;

const SCENARIO: &str = "bibtex/word";

#[tokio::test]
async fn no_text() {
    run_empty(SCENARIO, "foo.bib", 0, 0).await;
}

#[tokio::test]
async fn before_brace_entry() {
    run_empty(SCENARIO, "foo.bib", 2, 14).await;
}

#[tokio::test]
async fn before_brace_comment() {
    run_empty(SCENARIO, "foo.bib", 6, 14).await;
}
