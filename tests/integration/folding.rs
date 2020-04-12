use indoc::indoc;
use texlab::test::{TestBedBuilder, PULL_CAPABILITIES};
use texlab_protocol::{FoldingRange, FoldingRangeKind};

#[tokio::test]
async fn empty_latex_document() {
    let mut test_bed = TestBedBuilder::new().file("main.tex", "").build().await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_foldings = test_bed.folding_range("main.tex").await.unwrap();

    test_bed.shutdown().await;

    assert!(actual_foldings.is_empty());
}

#[tokio::test]
async fn empty_bibtex_document() {
    let mut test_bed = TestBedBuilder::new().file("main.bib", "").build().await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let actual_foldings = test_bed.folding_range("main.bib").await.unwrap();

    test_bed.shutdown().await;

    assert!(actual_foldings.is_empty());
}

#[tokio::test]
async fn latex_sections_with_env() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \begin{document}
                    \section{Foo}
                    Foo
                    \section{Bar}
                    Bar
                    \end{document}
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_foldings = test_bed.folding_range("main.tex").await.unwrap();

    test_bed.shutdown().await;

    let expected_foldings = vec![
        FoldingRange {
            start_line: 0,
            start_character: Some(16),
            end_line: 5,
            end_character: Some(0),
            kind: Some(FoldingRangeKind::Region),
        },
        FoldingRange {
            start_line: 1,
            start_character: Some(13),
            end_line: 2,
            end_character: Some(0),
            kind: Some(FoldingRangeKind::Region),
        },
    ];
    assert_eq!(actual_foldings, expected_foldings);
}

#[tokio::test]
async fn bibtex_entry() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.bib",
            indoc!(
                r#"
                    @article{foo,
                        author = {Foo Bar},
                        title = {Baz Qux},
                    }
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let actual_foldings = test_bed.folding_range("main.bib").await.unwrap();

    test_bed.shutdown().await;

    let expected_foldings = vec![FoldingRange {
        start_line: 0,
        start_character: Some(0),
        end_line: 3,
        end_character: Some(1),
        kind: Some(FoldingRangeKind::Region),
    }];
    assert_eq!(actual_foldings, expected_foldings);
}

#[tokio::test]
async fn unknown_file() {
    let mut test_bed = TestBedBuilder::new().build().await;

    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;

    let actual_foldings = test_bed.folding_range("main.tex").await;

    test_bed.shutdown().await;

    assert_eq!(actual_foldings, None);
}
