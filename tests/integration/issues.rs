use indoc::indoc;
use texlab::test::{TestBedBuilder, LOCATION_LINK_CAPABILITIES, PULL_CAPABILITIES};
use texlab_protocol::{
    HoverContents, Location, LocationLink, MarkupContent, MarkupKind, Range, RangeExt,
};

#[tokio::test]
async fn issue_14() {
    let mut test_bed = TestBedBuilder::new()
        .file("main.tex", r#"\(\be\)"#)
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_items = test_bed.completion("main.tex", 0, 5).await.unwrap();

    test_bed.shutdown().await;
    assert!(actual_items.iter().any(|item| item.label == "beta"));
}

#[tokio::test]
async fn issue_15_link() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    \begin{document}
                    \newcommand{\test}{test}
                    hello \test{}  
                    \end{document}
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed
        .initialize(LOCATION_LINK_CAPABILITIES.clone())
        .await;
    test_bed.open("main.tex").await;

    let actual_links = test_bed.definition_link("main.tex", 3, 9).await.unwrap();

    test_bed.shutdown().await;
    let expected_links = vec![LocationLink {
        origin_selection_range: Some(Range::new_simple(3, 6, 3, 13)),
        target_range: Range::new_simple(2, 0, 2, 24),
        target_selection_range: Range::new_simple(2, 0, 2, 24),
        target_uri: test_bed.uri("main.tex").into(),
    }];
    assert_eq!(actual_links, expected_links);
}

#[tokio::test]
async fn issue_15_location() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    \begin{document}
                    \newcommand{\test}{test}
                    hello \test{}  
                    \end{document}
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_locations = test_bed
        .definition_location("main.tex", 3, 9)
        .await
        .unwrap();

    test_bed.shutdown().await;
    let expected_locations = vec![Location {
        range: Range::new_simple(2, 0, 2, 24),
        uri: test_bed.uri("main.tex").into(),
    }];
    assert_eq!(actual_locations, expected_locations);
}

#[tokio::test]
async fn issue_17() {
    let mut test_bed = TestBedBuilder::new()
        .file("main.bib", r#"@ARTICLE{foo,}"#)
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let actual_hover = test_bed.hover("main.bib", 0, 3).await.unwrap().unwrap();

    test_bed.shutdown().await;
    assert_eq!(actual_hover.range.unwrap(), Range::new_simple(0, 0, 0, 8));
}

#[tokio::test]
async fn issue_21() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \label{foo}
                    \cref{}
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_items = test_bed.completion("main.tex", 1, 6).await.unwrap();

    test_bed.shutdown().await;
    assert!(actual_items.iter().any(|item| item.label == "foo"));
}

#[tokio::test]
async fn issue_22_include() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                \bibliography{bibfile}
                \cite{}
            "#
            ),
        )
        .file("bibfile.bib", r#"@article{foo,}"#)
        .build()
        .await;

    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_items = test_bed.completion("main.tex", 1, 6).await.unwrap();

    test_bed.shutdown().await;
    assert!(actual_items.iter().any(|item| item.label == "foo"));
}

#[tokio::test]
async fn issue_22_definition() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \bibliography{bibfile}
                    \cite{A,B}
                "#
            ),
        )
        .file(
            "bibfile.bib",
            indoc!(
                r#"
                    @article{A,}
                    @article{B,}
                "#
            ),
        )
        .build()
        .await;

    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_locations = test_bed
        .definition_location("main.tex", 1, 8)
        .await
        .unwrap();

    test_bed.shutdown().await;
    let expected_locations = vec![Location {
        range: Range::new_simple(1, 9, 1, 10),
        uri: test_bed.uri("bibfile.bib").into(),
    }];
    assert_eq!(actual_locations, expected_locations);
}

#[tokio::test]
async fn issue_23_completion() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    \begin{document}
                    \include{test1}
                    \include{test2}
                    \end{document}
                "#
            ),
        )
        .file("test1.tex", r#"\section{Section 1}\label{sec:1}"#)
        .file(
            "test2.tex",
            indoc!(
                r#"
                    \section{Section 2}\label{sec:2}
                    %
                    This section continues from Section \ref{sec:1}"#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("test2.tex").await;
    test_bed.detect_root("test2.tex").await;

    let actual_items = test_bed.completion("test2.tex", 2, 42).await.unwrap();

    test_bed.shutdown().await;
    assert_eq!(actual_items.len(), 2);
}

#[tokio::test]
async fn issue_23_rename() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    \begin{document}
                    \include{test1}
                    \include{test2}
                    \end{document}
                "#
            ),
        )
        .file("test1.tex", r#"\section{Section 1}\label{sec:1}"#)
        .file(
            "test2.tex",
            indoc!(
                r#"
                    \section{Section 2}\label{sec:2}
                    %
                    This section continues from Section \ref{sec:1}"#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("test1.tex").await;
    test_bed.detect_root("test1.tex").await;

    let workspace_edit = test_bed
        .rename("test1.tex", 0, 27, "foo")
        .await
        .unwrap()
        .unwrap();

    test_bed.shutdown().await;
    let changes = workspace_edit.changes.unwrap();
    assert!(changes.contains_key(&test_bed.uri("test1.tex")));
    assert!(changes.contains_key(&test_bed.uri("test2.tex")));
}

#[tokio::test]
async fn issue_23_hover() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    \begin{document}
                    \include{test1}
                    \include{test2}
                    \end{document}
                "#
            ),
        )
        .file("test1.tex", r#"\section{Section 1}\label{sec:1}"#)
        .file(
            "test2.tex",
            indoc!(
                r#"
                    \section{Section 2}\label{sec:2}
                    %
                    This section continues from Section \ref{sec:1}"#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("test2.tex").await;
    test_bed.detect_root("test2.tex").await;

    let actual_hover = test_bed.hover("test2.tex", 2, 42).await.unwrap().unwrap();

    test_bed.shutdown().await;
    assert_eq!(
        actual_hover.contents,
        HoverContents::Markup(MarkupContent {
            kind: MarkupKind::PlainText,
            value: "Section (Section 1)".into()
        })
    );
}

#[tokio::test]
async fn issue_26() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                \section{Foo}\label{sec:foo}
                \begin{equation}\label{eq:bar}
                \end{equation}
                \eqref{}
            "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_items = test_bed.completion("main.tex", 3, 7).await.unwrap();

    test_bed.shutdown().await;
    assert!(actual_items.iter().any(|item| item.label == "eq:bar"));
    assert!(actual_items.iter().all(|item| item.label != "sec:foo"));
}
