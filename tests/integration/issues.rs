use indoc::indoc;
use texlab::{
    protocol::{HoverContents, MarkupContent, MarkupKind},
    test::{TestBedBuilder, PULL_CAPABILITIES},
};

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
    assert_eq!(
        actual_hover.contents,
        HoverContents::Markup(MarkupContent {
            kind: MarkupKind::PlainText,
            value: "Section (Section 1)".into()
        })
    );
}
