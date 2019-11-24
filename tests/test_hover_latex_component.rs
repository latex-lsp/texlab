pub mod support;

use support::hover::*;

const SCENARIO: &str = "latex/component";

#[tokio::test]
async fn class_known() {
    run(SCENARIO, "foo.tex", 0, 18).await.unwrap();
}

#[tokio::test]
async fn class_unknown() {
    let contents = run(SCENARIO, "foo.tex", 2, 16).await;
    assert_eq!(contents, None);
}

#[tokio::test]
async fn package_known() {
    run(SCENARIO, "foo.tex", 1, 17).await.unwrap();
}

#[tokio::test]
async fn package_unknown() {
    let contents = run(SCENARIO, "foo.tex", 3, 14).await;
    assert_eq!(contents, None);
}
