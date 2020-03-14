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

    let actual_refs = test_bed.references("main.tex", 0, 0, false).await.unwrap();

    test_bed.shutdown().await;

    assert!(actual_refs.is_empty());
}

#[tokio::test]
async fn empty_bibtex_document() {
    let mut test_bed = TestBedBuilder::new().file("main.bib", "").build().await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let actual_refs = test_bed.references("main.bib", 0, 0, false).await.unwrap();

    test_bed.shutdown().await;

    assert!(actual_refs.is_empty());
}

#[tokio::test]
async fn bibtex_entry() {
    let mut test_bed = TestBedBuilder::new()
        .file("main.bib", r#"@article{foo,}"#)
        .file(
            "main.tex",
            indoc!(
                r#"
                    \addbibresource{main.bib}
                    \cite{foo}
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;
    test_bed.open("main.tex").await;

    let actual_refs = test_bed.references("main.tex", 1, 8, true).await.unwrap();

    test_bed.shutdown().await;

    let expected_refs = vec![
        Location {
            uri: test_bed.uri("main.tex").into(),
            range: Range::new_simple(1, 6, 1, 9),
        },
        Location {
            uri: test_bed.uri("main.bib").into(),
            range: Range::new_simple(0, 9, 0, 12),
        },
    ];

    assert_eq!(actual_refs, expected_refs);
}

#[tokio::test]
async fn bibtex_string() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.bib",
            indoc!(
                r#"
                    @string{foo = "foo"}
                    @article{bar, author = foo # foo}
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let actual_refs = test_bed.references("main.bib", 1, 31, false).await.unwrap();

    test_bed.shutdown().await;

    let expected_refs = vec![
        Location {
            uri: test_bed.uri("main.bib").into(),
            range: Range::new_simple(1, 23, 1, 26),
        },
        Location {
            uri: test_bed.uri("main.bib").into(),
            range: Range::new_simple(1, 29, 1, 32),
        },
    ];

    assert_eq!(actual_refs, expected_refs);
}

#[tokio::test]
async fn latex_label() {
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

    let actual_refs = test_bed.references("main.tex", 0, 7, false).await.unwrap();

    test_bed.shutdown().await;

    let expected_refs = vec![Location {
        uri: test_bed.uri("main.tex").into(),
        range: Range::new_simple(1, 5, 1, 8),
    }];

    assert_eq!(actual_refs, expected_refs);
}

#[tokio::test]
async fn unknown_file() {
    let mut test_bed = TestBedBuilder::new().build().await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;

    let actual_refs = test_bed.references("main.tex", 0, 0, false).await;

    test_bed.shutdown().await;

    assert_eq!(actual_refs, None);
}
