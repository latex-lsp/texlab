use crate::{
    completion::types::{Item, ItemData},
    components::COMPONENT_DATABASE,
    feature::FeatureRequest,
    protocol::{CompletionParams, RangeExt},
    syntax::SyntaxNode,
    workspace::DocumentContent,
};

pub async fn complete_bibtex_commands<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
) {
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
                for cmd in &COMPONENT_DATABASE.kernel().commands {
                    let item = Item::new(
                        range,
                        ItemData::ComponentCommand {
                            name: &cmd.name,
                            image: cmd.image.as_deref(),
                            glyph: cmd.glyph.as_deref(),
                            file_names: &[],
                        },
                    );
                    items.push(item);
                }
            }
        }
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

        complete_bibtex_commands(&req, &mut actual_items).await;

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

        complete_bibtex_commands(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_comment() {
        let req = FeatureTester::new()
            .file("main.bib", r#"\"#)
            .main("main.bib")
            .position(0, 1)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_bibtex_commands(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_command() {
        let req = FeatureTester::new()
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
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_bibtex_commands(&req, &mut actual_items).await;

        assert!(!actual_items.is_empty());
        assert_eq!(actual_items[0].range, Range::new_simple(1, 1, 1, 2));
    }

    #[tokio::test]
    async fn start_of_command() {
        let req = FeatureTester::new()
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
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_bibtex_commands(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_text() {
        let req = FeatureTester::new()
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
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_bibtex_commands(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_latex_command() {
        let req = FeatureTester::new()
            .file("main.tex", r#"\"#)
            .main("main.tex")
            .position(0, 1)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_bibtex_commands(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }
}
