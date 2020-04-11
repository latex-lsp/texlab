use indoc::indoc;
use texlab::{
    protocol::{HoverContents, MarkupContent, MarkupKind},
    test::{TestBedBuilder, PULL_CAPABILITIES},
};

#[tokio::test]
async fn label_theorem_child_file() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    \newtheorem{lemma}{Lemma}
                    \include{child}
                    \ref{thm:foo}
                "#
            ),
        )
        .file(
            "child.tex",
            indoc!(
                r#"
                    \begin{lemma}\label{thm:foo}
                        1 + 1 = 2
                    \end{lemma}
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_hover = test_bed.hover("main.tex", 3, 8).await.unwrap().unwrap();
    assert_eq!(
        actual_hover.contents,
        HoverContents::Markup(MarkupContent {
            kind: MarkupKind::PlainText,
            value: "Lemma".into()
        })
    );
}

#[tokio::test]
async fn label_theorem_child_file_number() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                \documentclass{article}
                \newtheorem{lemma}{Lemma}
                \include{child}
                \ref{thm:foo}
            "#
            ),
        )
        .file(
            "child.tex",
            indoc!(
                r#"
                \begin{lemma}[Foo]\label{thm:foo}
                    1 + 1 = 2
                \end{lemma}
            "#
            ),
        )
        .file("child.aux", r#"\newlabel{thm:foo}{{1}{1}{Foo}{lemma.1}{}}"#)
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_hover = test_bed.hover("main.tex", 3, 8).await.unwrap().unwrap();
    assert_eq!(
        actual_hover.contents,
        HoverContents::Markup(MarkupContent {
            kind: MarkupKind::PlainText,
            value: "Lemma 1 (Foo)".into()
        })
    );
}
