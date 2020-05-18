use crate::{
    completion::types::{Item, ItemData},
    feature::FeatureRequest,
    protocol::{CompletionParams, Range, RangeExt},
    syntax::{bibtex, SyntaxNode, LANGUAGE_DATA},
    workspace::DocumentContent,
};

pub async fn complete_bibtex_fields<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
) {
    if let DocumentContent::Bibtex(tree) = &req.current().content {
        let pos = req.params.text_document_position.position;
        match tree
            .find(pos)
            .into_iter()
            .last()
            .map(|node| &tree.graph[node])
        {
            Some(bibtex::Node::Field(field)) => {
                if field.name.range().contains(pos) {
                    make_items(items, field.name.range());
                    return;
                }
            }
            Some(bibtex::Node::Entry(entry)) => {
                if !entry.is_comment() && !entry.ty.range().contains(pos) {
                    let range = Range::new(pos, pos);
                    if let Some(key) = &entry.key {
                        if !key.range().contains(pos) {
                            make_items(items, range);
                            return;
                        }
                    } else {
                        make_items(items, range);
                        return;
                    }
                }
            }
            _ => (),
        }
    }
}

fn make_items(items: &mut Vec<Item>, range: Range) {
    for field in &LANGUAGE_DATA.fields {
        let item = Item::new(range, ItemData::Field { field });
        items.push(item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{feature::FeatureTester, protocol::Range};
    use indoc::indoc;

    #[tokio::test]
    async fn empty_latex_document() {
        let req = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_bibtex_fields(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let req = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_bibtex_fields(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_first_field() {
        let req = FeatureTester::new()
            .file(
                "main.bib",
                indoc!(
                    r#"
                        @article{foo,
                        bar}
                    "#
                ),
            )
            .main("main.bib")
            .position(1, 1)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_bibtex_fields(&req, &mut actual_items).await;

        assert!(!actual_items.is_empty());
        assert_eq!(actual_items[0].range, Range::new_simple(1, 0, 1, 3));
    }

    #[tokio::test]
    async fn inside_second_field() {
        let req = FeatureTester::new()
            .file("main.bib", "@article{foo, bar = {baz}, qux}")
            .main("main.bib")
            .position(0, 27)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_bibtex_fields(&req, &mut actual_items).await;

        assert!(!actual_items.is_empty());
        assert_eq!(actual_items[0].range, Range::new_simple(0, 27, 0, 30));
    }

    #[tokio::test]
    async fn inside_entry() {
        let req = FeatureTester::new()
            .file(
                "main.bib",
                indoc!(
                    r#"
                        @article{foo,
                        }
                    "#
                ),
            )
            .main("main.bib")
            .position(1, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_bibtex_fields(&req, &mut actual_items).await;

        assert!(!actual_items.is_empty());
        assert_eq!(actual_items[0].range, Range::new_simple(1, 0, 1, 0));
    }

    #[tokio::test]
    async fn inside_content() {
        let req = FeatureTester::new()
            .file(
                "main.bib",
                indoc!(
                    r#"
                        @article{foo,
                        bar = {baz}}
                    "#
                ),
            )
            .main("main.bib")
            .position(1, 7)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_bibtex_fields(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_entry_type() {
        let req = FeatureTester::new()
            .file("main.bib", "@article{foo,}")
            .main("main.bib")
            .position(0, 3)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_bibtex_fields(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }
}
