use crate::factory::{self, LatexComponentId};
use async_trait::async_trait;
use texlab_components::COMPONENT_DATABASE;
use texlab_feature::{DocumentContent, FeatureProvider, FeatureRequest};
use texlab_protocol::{CompletionItem, CompletionParams, RangeExt, TextEdit};
use texlab_syntax::SyntaxNode;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct BibtexCommandCompletionProvider;

#[async_trait]
impl FeatureProvider for BibtexCommandCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut items = Vec::new();
        if let DocumentContent::Bibtex(tree) = &req.current().content {
            let pos = req.params.text_document_position.position;
            if let Some(cmd) = tree
                .find(pos)
                .into_iter()
                .last()
                .and_then(|node| tree.as_command(node))
            {
                if cmd.token.range().contains(pos) && cmd.token.start().character != pos.character {
                    let mut range = cmd.range();
                    range.start.character += 1;

                    let component = LatexComponentId::kernel();
                    for cmd in &COMPONENT_DATABASE.kernel().commands {
                        let text_edit = TextEdit::new(range, (&cmd.name).into());
                        let item = factory::command(
                            req,
                            (&cmd.name).into(),
                            cmd.image.as_ref().map(AsRef::as_ref),
                            cmd.glyph.as_ref().map(AsRef::as_ref),
                            text_edit,
                            &component,
                        );
                        items.push(item);
                    }
                }
            }
        }
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use texlab_feature::FeatureTester;
    use texlab_protocol::{CompletionTextEditExt, Range};

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_items = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion(BibtexCommandCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion(BibtexCommandCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_comment() {
        let actual_items = FeatureTester::new()
            .file("main.bib", r#"\"#)
            .main("main.bib")
            .position(0, 1)
            .test_completion(BibtexCommandCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_command() {
        let actual_items = FeatureTester::new()
            .file(
                "main.bib",
                indoc!(
                    r#"
                        @article{foo, bar=
                        \}
                    "#
                ),
            )
            .main("main.bib")
            .position(1, 1)
            .test_completion(BibtexCommandCompletionProvider)
            .await;

        assert!(!actual_items.is_empty());
        assert_eq!(
            actual_items[0]
                .text_edit
                .as_ref()
                .and_then(|edit| edit.text_edit())
                .map(|edit| edit.range)
                .unwrap(),
            Range::new_simple(1, 1, 1, 2)
        );
    }

    #[tokio::test]
    async fn start_of_command() {
        let actual_items = FeatureTester::new()
            .file(
                "main.bib",
                indoc!(
                    r#"
                        @article{foo, bar=
                        \}
                    "#
                ),
            )
            .main("main.bib")
            .position(1, 0)
            .test_completion(BibtexCommandCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_text() {
        let actual_items = FeatureTester::new()
            .file(
                "main.bib",
                indoc!(
                    r#"
                        @article{foo, bar=
                        }
                    "#
                ),
            )
            .main("main.bib")
            .position(1, 0)
            .test_completion(BibtexCommandCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_latex_command() {
        let actual_items = FeatureTester::new()
            .file("main.tex", r#"\"#)
            .main("main.tex")
            .position(0, 1)
            .test_completion(BibtexCommandCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }
}
