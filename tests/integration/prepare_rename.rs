use texlab::test::{TestBedBuilder, PULL_CAPABILITIES};
use texlab_protocol::{Range, RangeExt};

#[tokio::test]
async fn empty_latex_document() {
    let mut test_bed = TestBedBuilder::new().file("main.tex", "").build().await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_range = test_bed.prepare_rename("main.tex", 0, 0).await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_range, None);
}

#[tokio::test]
async fn empty_bibtex_document() {
    let mut test_bed = TestBedBuilder::new().file("main.bib", "").build().await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let actual_range = test_bed.prepare_rename("main.bib", 0, 0).await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_range, None);
}

#[tokio::test]
async fn bibtex_entry() {
    let mut test_bed = TestBedBuilder::new()
        .file("main.bib", r#"@article{foo,}"#)
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let actual_range = test_bed
        .prepare_rename("main.bib", 0, 10)
        .await
        .unwrap()
        .unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_range, Range::new_simple(0, 9, 0, 12));
}

#[tokio::test]
async fn latex_citation() {
    let mut test_bed = TestBedBuilder::new()
        .file("main.tex", r#"\cite{foo,bar}"#)
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_range = test_bed
        .prepare_rename("main.tex", 0, 11)
        .await
        .unwrap()
        .unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_range, Range::new_simple(0, 10, 0, 13));
}

#[tokio::test]
async fn latex_command() {
    let mut test_bed = TestBedBuilder::new()
        .file("main.tex", r#"\foo"#)
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_range = test_bed
        .prepare_rename("main.tex", 0, 1)
        .await
        .unwrap()
        .unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_range, Range::new_simple(0, 0, 0, 4));
}

#[tokio::test]
async fn latex_environment() {
    let mut test_bed = TestBedBuilder::new()
        .file("main.tex", r#"\begin{foo}\end{bar}"#)
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_range = test_bed
        .prepare_rename("main.tex", 0, 7)
        .await
        .unwrap()
        .unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_range, Range::new_simple(0, 7, 0, 10));
}

#[tokio::test]
async fn latex_label() {
    let mut test_bed = TestBedBuilder::new()
        .file("main.tex", r#"\ref{foo,bar}"#)
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_range = test_bed
        .prepare_rename("main.tex", 0, 9)
        .await
        .unwrap()
        .unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_range, Range::new_simple(0, 9, 0, 12));
}

#[tokio::test]
async fn unknown_file() {
    let mut test_bed = TestBedBuilder::new().build().await;

    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;

    let actual_range = test_bed.prepare_rename("main.tex", 0, 0).await;

    test_bed.shutdown().await;

    assert_eq!(actual_range, None);
}
