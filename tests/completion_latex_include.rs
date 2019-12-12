use texlab_test::completion::*;

const SCENARIO: &str = "latex/include";

#[tokio::test]
async fn include_root() {
    let (_, items) = run_list(SCENARIO, "foo.tex", 2, 9).await;
    verify::labels(&items, vec!["bar", "foo", "qux"]);
}

#[tokio::test]
async fn input_root() {
    let (_, items) = run_list(SCENARIO, "foo.tex", 3, 7).await;
    verify::labels(&items, vec!["bar.tex", "foo.tex", "qux"]);
}

#[tokio::test]
async fn input_subdirectory() {
    let (_, items) = run_list(SCENARIO, "foo.tex", 4, 11).await;
    verify::labels(&items, vec!["baz.tex"]);
}

#[tokio::test]
async fn bibliography() {
    let (_, items) = run_list(SCENARIO, "foo.tex", 5, 16).await;
    verify::labels(&items, vec!["bibliography.bib", "qux"]);
}
