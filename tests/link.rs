use indoc::indoc;
use texlab::{
    protocol::{DocumentLink, Range, RangeExt},
    test::{TestBedBuilder, PULL_CAPABILITIES, PUSH_CAPABILITIES},
};

#[tokio::test]
async fn empty_latex_document() {
    let mut test_bed = TestBedBuilder::new().file("main.tex", "").build().await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_links = test_bed.document_link("main.tex").await.unwrap();

    test_bed.shutdown().await;

    assert!(actual_links.is_empty());
}

#[tokio::test]
async fn default_settings() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \include{foo/bar}
                    \input{qux.tex}
                "#
            ),
        )
        .file("foo/bar.tex", "")
        .file("qux.tex", "")
        .build()
        .await;

    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_links = test_bed.document_link("main.tex").await.unwrap();

    test_bed.shutdown().await;

    let expected_links = vec![
        DocumentLink {
            range: Range::new_simple(0, 9, 0, 16),
            target: test_bed.uri("foo/bar.tex").into(),
            tooltip: None,
        },
        DocumentLink {
            range: Range::new_simple(1, 7, 1, 14),
            target: test_bed.uri("qux.tex").into(),
            tooltip: None,
        },
    ];
    assert_eq!(actual_links, expected_links);
}

#[tokio::test]
async fn root_directory() {
    let mut test_bed = TestBedBuilder::new()
        .file("src/main.tex", r#"\include{src/foo}"#)
        .file("src/foo.tex", "")
        .root_dir(".")
        .build()
        .await;

    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("src/main.tex").await;

    let actual_links = test_bed.document_link("src/main.tex").await.unwrap();

    test_bed.shutdown().await;

    let expected_links = vec![DocumentLink {
        range: Range::new_simple(0, 9, 0, 16),
        target: test_bed.uri("src/foo.tex").into(),
        tooltip: None,
    }];
    assert_eq!(actual_links, expected_links);
}

#[tokio::test]
async fn parent_directory() {
    let mut test_bed = TestBedBuilder::new()
        .file("src/main.tex", r#"\addbibresource{../foo.bib}"#)
        .file("foo.bib", "")
        .build()
        .await;

    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("src/main.tex").await;

    let actual_links = test_bed.document_link("src/main.tex").await.unwrap();

    test_bed.shutdown().await;

    let expected_links = vec![DocumentLink {
        range: Range::new_simple(0, 16, 0, 26),
        target: test_bed.uri("foo.bib").into(),
        tooltip: None,
    }];
    assert_eq!(actual_links, expected_links);
}

#[tokio::test]
async fn unknown_file() {
    let mut test_bed = TestBedBuilder::new().build().await;

    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;

    let actual_links = test_bed.document_link("main.tex").await;

    test_bed.shutdown().await;

    assert_eq!(actual_links, None);
}

#[tokio::test]
async fn edit_file() {
    let mut test_bed = TestBedBuilder::new()
        .file("main.tex", "")
        .file("foo.tex", "")
        .build()
        .await;

    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;
    test_bed.edit("main.tex", r#"\include{foo}"#).await;

    let actual_links = test_bed.document_link("main.tex").await.unwrap();

    test_bed.shutdown().await;

    let expected_links = vec![DocumentLink {
        range: Range::new_simple(0, 9, 0, 12),
        target: test_bed.uri("foo.tex").into(),
        tooltip: None,
    }];
    assert_eq!(actual_links, expected_links);
}

#[tokio::test]
async fn did_change_configuration() {
    let mut test_bed = TestBedBuilder::new()
        .file("src/main.tex", r#"\include{src/foo}"#)
        .file("src/foo.tex", "")
        .root_dir(".")
        .build()
        .await;

    test_bed.spawn();
    test_bed.initialize(PUSH_CAPABILITIES.clone()).await;
    test_bed.open("src/main.tex").await;
    test_bed.push_options().await;

    let actual_links = test_bed.document_link("src/main.tex").await.unwrap();

    test_bed.shutdown().await;

    let expected_links = vec![DocumentLink {
        range: Range::new_simple(0, 9, 0, 16),
        target: test_bed.uri("src/foo.tex").into(),
        tooltip: None,
    }];
    assert_eq!(actual_links, expected_links);
}
