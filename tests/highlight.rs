use indoc::indoc;
use texlab::{
    protocol::*,
    test::{TestBedBuilder, PULL_CAPABILITIES},
};

#[tokio::test]
async fn empty_latex_document() {
    let mut test_bed = TestBedBuilder::new().file("main.tex", "").build().await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_highlights = test_bed.document_highlight("main.tex", 0, 0).await.unwrap();

    test_bed.shutdown().await;

    assert!(actual_highlights.is_empty());
}

#[tokio::test]
async fn label() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \label{foo}
                    \ref{foo}
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_highlights = test_bed.document_highlight("main.tex", 0, 7).await.unwrap();

    let expected_highlights = vec![
        DocumentHighlight {
            range: Range::new_simple(0, 7, 0, 10),
            kind: Some(DocumentHighlightKind::Write),
        },
        DocumentHighlight {
            range: Range::new_simple(1, 5, 1, 8),
            kind: Some(DocumentHighlightKind::Read),
        },
    ];

    test_bed.shutdown().await;

    assert_eq!(actual_highlights, expected_highlights);
}

#[tokio::test]
async fn unknown_file() {
    let mut test_bed = TestBedBuilder::new().build().await;

    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;

    let actual_highlights = test_bed.document_highlight("main.tex", 0, 0).await;

    test_bed.shutdown().await;

    assert_eq!(actual_highlights, None);
}
