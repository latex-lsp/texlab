use indoc::indoc;
use std::collections::HashMap;
use texlab::{
    protocol::{Range, RangeExt, TextEdit, WorkspaceEdit},
    test::{TestBedBuilder, PULL_CAPABILITIES},
};

#[tokio::test]
async fn empty_latex_document() {
    let mut test_bed = TestBedBuilder::new().file("main.tex", "").build().await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_edit = test_bed.rename("main.tex", 0, 0, "").await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_edit, None);
}

#[tokio::test]
async fn empty_bibtex_document() {
    let mut test_bed = TestBedBuilder::new().file("main.bib", "").build().await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let actual_edit = test_bed.rename("main.bib", 0, 0, "").await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_edit, None);
}

#[tokio::test]
async fn bibtex_entry() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.bib",
            indoc!(
                r#"
                    @article{foo,}
                    @article{bar,}
                "#
            ),
        )
        .file(
            "main.tex",
            indoc!(
                r#"
                    \addbibresource{main.bib}
                    \cite{foo,bar}
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;
    test_bed.open("main.tex").await;

    let actual_edit = test_bed
        .rename("main.bib", 1, 10, "baz")
        .await
        .unwrap()
        .unwrap();

    test_bed.shutdown().await;

    let mut expected_changes = HashMap::new();
    expected_changes.insert(
        test_bed.uri("main.bib").into(),
        vec![TextEdit::new(Range::new_simple(1, 9, 1, 12), "baz".into())],
    );
    expected_changes.insert(
        test_bed.uri("main.tex").into(),
        vec![TextEdit::new(Range::new_simple(1, 10, 1, 13), "baz".into())],
    );

    assert_eq!(actual_edit, WorkspaceEdit::new(expected_changes));
}

#[tokio::test]
async fn latex_command() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "foo.tex",
            indoc!(
                r#"
                    \input{bar.tex}
                    \foo
                "#
            ),
        )
        .file(
            "bar.tex",
            indoc!(
                r#"
                    \foo
                    \bar
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("foo.tex").await;
    test_bed.open("bar.tex").await;

    let actual_edit = test_bed
        .rename("foo.tex", 1, 2, "baz")
        .await
        .unwrap()
        .unwrap();

    test_bed.shutdown().await;

    let mut expected_changes = HashMap::new();
    expected_changes.insert(
        test_bed.uri("foo.tex").into(),
        vec![TextEdit::new(Range::new_simple(1, 0, 1, 4), "\\baz".into())],
    );
    expected_changes.insert(
        test_bed.uri("bar.tex").into(),
        vec![TextEdit::new(Range::new_simple(0, 0, 0, 4), "\\baz".into())],
    );

    assert_eq!(actual_edit, WorkspaceEdit::new(expected_changes));
}

#[tokio::test]
async fn latex_environment() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \begin{foo}
                        \begin{bar}
                            Baz
                        \end{bar}
                    \end{foo}
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_edit = test_bed
        .rename("main.tex", 3, 11, "baz")
        .await
        .unwrap()
        .unwrap();

    test_bed.shutdown().await;

    let mut expected_changes = HashMap::new();
    expected_changes.insert(
        test_bed.uri("main.tex").into(),
        vec![
            TextEdit::new(Range::new_simple(1, 11, 1, 14), "baz".into()),
            TextEdit::new(Range::new_simple(3, 9, 3, 12), "baz".into()),
        ],
    );

    assert_eq!(actual_edit, WorkspaceEdit::new(expected_changes));
}

#[tokio::test]
async fn latex_label() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "foo.tex",
            indoc!(
                r#"
                    \input{bar.tex}
                    \ref{foo,bar}
                "#
            ),
        )
        .file(
            "bar.tex",
            indoc!(
                r#"
                    \label{foo}
                    \label{bar}
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("foo.tex").await;
    test_bed.open("bar.tex").await;

    let actual_edit = test_bed
        .rename("foo.tex", 1, 10, "baz")
        .await
        .unwrap()
        .unwrap();

    test_bed.shutdown().await;

    let mut expected_changes = HashMap::new();
    expected_changes.insert(
        test_bed.uri("foo.tex").into(),
        vec![TextEdit::new(Range::new_simple(1, 9, 1, 12), "baz".into())],
    );
    expected_changes.insert(
        test_bed.uri("bar.tex").into(),
        vec![TextEdit::new(Range::new_simple(1, 7, 1, 10), "baz".into())],
    );

    assert_eq!(actual_edit, WorkspaceEdit::new(expected_changes));
}

#[tokio::test]
async fn unknown_file() {
    let mut test_bed = TestBedBuilder::new().build().await;

    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;

    let actual_edit = test_bed.rename("main.tex", 0, 0, "").await;

    test_bed.shutdown().await;

    assert_eq!(actual_edit, None);
}
