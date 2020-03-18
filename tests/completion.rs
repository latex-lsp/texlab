use indoc::indoc;
use texlab::{
    protocol::{CompletionItem, Range, RangeExt, TextEdit},
    test::{TestBed, TestBedBuilder, TestLspClient, PULL_CAPABILITIES},
};

async fn run_item(
    test_bed: &TestBed,
    relative_path: &str,
    line: u64,
    character: u64,
    label: &str,
) -> CompletionItem {
    let item = test_bed
        .completion(relative_path, line, character)
        .await
        .unwrap()
        .into_iter()
        .find(|item| item.label == label)
        .unwrap();

    test_bed.client.completion_resolve(item).await.unwrap()
}

fn verify_text_edit(
    item: &CompletionItem,
    start_line: u64,
    start_character: u64,
    end_line: u64,
    end_character: u64,
    text: &str,
) {
    assert_eq!(
        *item.text_edit.as_ref().unwrap(),
        TextEdit::new(
            Range::new_simple(start_line, start_character, end_line, end_character),
            text.into()
        )
    );
}

fn verify_detail(item: &CompletionItem, detail: &str) {
    assert_eq!(item.detail.as_ref().unwrap(), detail);
}

#[tokio::test]
async fn empty_latex_document() {
    let mut test_bed = TestBedBuilder::new().file("main.tex", "").build().await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_items = test_bed.completion("main.tex", 0, 0).await.unwrap();

    test_bed.shutdown().await;

    assert!(actual_items.is_empty());
}

#[tokio::test]
async fn empty_bibtex_document() {
    let mut test_bed = TestBedBuilder::new().file("main.bib", "").build().await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let actual_items = test_bed.completion("main.bib", 0, 0).await.unwrap();

    test_bed.shutdown().await;

    assert!(actual_items.is_empty());
}

#[tokio::test]
async fn bibtex_comment() {
    let mut test_bed = TestBedBuilder::new().file("main.bib", "foo").build().await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let actual_items = test_bed.completion("main.bib", 0, 2).await.unwrap();

    test_bed.shutdown().await;

    assert!(actual_items.is_empty());
}

#[tokio::test]
async fn bibtex_command_incomplete_entry() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.bib",
            indoc!(
                r#"
                    @article{foo,
                        author = {\LaT
                    }
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let item = run_item(&test_bed, "main.bib", 1, 18, "LaTeX").await;

    test_bed.shutdown().await;

    verify_detail(&item, "built-in");
    verify_text_edit(&item, 1, 15, 1, 18, "LaTeX");
}

#[tokio::test]
async fn bibtex_command_complete_entry() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.bib",
            indoc!(
                r#"
                    @article{foo,
                        author = {\LaT}
                    }
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let item = run_item(&test_bed, "main.bib", 1, 18, "LaTeX").await;

    test_bed.shutdown().await;

    verify_detail(&item, "built-in");
    verify_text_edit(&item, 1, 15, 1, 18, "LaTeX");
}

#[tokio::test]
async fn bibtex_type_empty() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.bib",
            indoc!(
                r#"
                    @
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let item = run_item(&test_bed, "main.bib", 0, 1, "article").await;

    test_bed.shutdown().await;

    assert!(item.documentation.is_some());
    verify_text_edit(&item, 0, 1, 0, 1, "article");
}

#[tokio::test]
async fn bibtex_type_incomplete() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.bib",
            indoc!(
                r#"
                    @art
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let item = run_item(&test_bed, "main.bib", 0, 1, "article").await;

    test_bed.shutdown().await;

    assert!(item.documentation.is_some());
    verify_text_edit(&item, 0, 1, 0, 4, "article");
}

#[tokio::test]
async fn bibtex_type_complete() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.bib",
            indoc!(
                r#"
                    @article
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let item = run_item(&test_bed, "main.bib", 0, 1, "article").await;

    test_bed.shutdown().await;

    assert!(item.documentation.is_some());
    verify_text_edit(&item, 0, 1, 0, 8, "article");
}

#[tokio::test]
async fn bibtex_field_incomplete_entry() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.bib",
            indoc!(
                r#"
                    @article{foo,
                        titl
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let item = run_item(&test_bed, "main.bib", 1, 6, "title").await;

    test_bed.shutdown().await;

    assert!(item.documentation.is_some());
    verify_text_edit(&item, 1, 4, 1, 8, "title");
}

#[tokio::test]
async fn bibtex_field_complete_entry() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.bib",
            indoc!(
                r#"
                    @article{foo,
                        title = {}
                    }
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let item = run_item(&test_bed, "main.bib", 1, 6, "title").await;

    test_bed.shutdown().await;

    assert!(item.documentation.is_some());
    verify_text_edit(&item, 1, 4, 1, 9, "title");
}
