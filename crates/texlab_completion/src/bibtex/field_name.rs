use crate::factory;
use futures_boxed::boxed;
use texlab_feature::{DocumentContent, FeatureProvider, FeatureRequest};
use texlab_protocol::{CompletionItem, CompletionParams, Range, RangeExt, TextEdit};
use texlab_syntax::{bibtex, SyntaxNode, LANGUAGE_DATA};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct BibtexFieldNameCompletionProvider;

impl FeatureProvider for BibtexFieldNameCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
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
                        return Self::make_items(req, field.name.range());
                    }
                }
                Some(bibtex::Node::Entry(entry)) => {
                    if !entry.is_comment() && !entry.ty.range().contains(pos) {
                        let edit_range = Range::new(pos, pos);
                        if let Some(key) = &entry.key {
                            if !key.range().contains(pos) {
                                return Self::make_items(req, edit_range);
                            }
                        } else {
                            return Self::make_items(req, edit_range);
                        }
                    }
                }
                _ => (),
            }
        }
        Vec::new()
    }
}

impl BibtexFieldNameCompletionProvider {
    fn make_items(
        req: &FeatureRequest<CompletionParams>,
        edit_range: Range,
    ) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        for field in &LANGUAGE_DATA.fields {
            let text_edit = TextEdit::new(edit_range, (&field.name).into());
            let item = factory::field_name(req, field, text_edit);
            items.push(item);
        }
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use texlab_feature::FeatureTester;
    use texlab_protocol::Range;

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_items = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion(BibtexFieldNameCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion(BibtexFieldNameCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_first_field() {
        let actual_items = FeatureTester::new()
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
            .test_completion(BibtexFieldNameCompletionProvider)
            .await;

        assert!(!actual_items.is_empty());
        assert_eq!(
            actual_items[0]
                .text_edit
                .as_ref()
                .map(|edit| edit.range)
                .unwrap(),
            Range::new_simple(1, 0, 1, 3)
        );
    }

    #[tokio::test]
    async fn inside_second_field() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "@article{foo, bar = {baz}, qux}")
            .main("main.bib")
            .position(0, 27)
            .test_completion(BibtexFieldNameCompletionProvider)
            .await;

        assert!(!actual_items.is_empty());
        assert_eq!(
            actual_items[0]
                .text_edit
                .as_ref()
                .map(|edit| edit.range)
                .unwrap(),
            Range::new_simple(0, 27, 0, 30)
        );
    }

    #[tokio::test]
    async fn inside_entry() {
        let actual_items = FeatureTester::new()
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
            .test_completion(BibtexFieldNameCompletionProvider)
            .await;

        assert!(!actual_items.is_empty());
        assert_eq!(
            actual_items[0]
                .text_edit
                .as_ref()
                .map(|edit| edit.range)
                .unwrap(),
            Range::new_simple(1, 0, 1, 0)
        );
    }

    #[tokio::test]
    async fn inside_content() {
        let actual_items = FeatureTester::new()
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
            .test_completion(BibtexFieldNameCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_entry_type() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "@article{foo,}")
            .main("main.bib")
            .position(0, 3)
            .test_completion(BibtexFieldNameCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }
}
