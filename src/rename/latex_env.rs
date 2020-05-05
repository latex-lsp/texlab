use crate::{
    feature::{FeatureProvider, FeatureRequest},
    protocol::{
        Position, Range, RangeExt, RenameParams, TextDocumentPositionParams, TextEdit,
        WorkspaceEdit,
    },
    syntax::{latex, SyntaxNode},
    workspace::DocumentContent,
};
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexEnvironmentPrepareRenameProvider;

#[async_trait]
impl FeatureProvider for LatexEnvironmentPrepareRenameProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Range>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let pos = req.params.position;
        let (left_name, right_name) = find_environment(&req.current().content, pos)?;
        let range = if left_name.range().contains(pos) {
            left_name.range()
        } else {
            right_name.range()
        };
        Some(range)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexEnvironmentRenameProvider;

#[async_trait]
impl FeatureProvider for LatexEnvironmentRenameProvider {
    type Params = RenameParams;
    type Output = Option<WorkspaceEdit>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let (left_name, right_name) = find_environment(
            &req.current().content,
            req.params.text_document_position.position,
        )?;
        let edits = vec![
            TextEdit::new(left_name.range(), req.params.new_name.clone()),
            TextEdit::new(right_name.range(), req.params.new_name.clone()),
        ];
        let mut changes = HashMap::new();
        changes.insert(req.current().uri.clone().into(), edits);
        Some(WorkspaceEdit::new(changes))
    }
}

fn find_environment(
    content: &DocumentContent,
    pos: Position,
) -> Option<(&latex::Token, &latex::Token)> {
    if let DocumentContent::Latex(table) = content {
        for env in &table.environments {
            if let Some(left_name) = env.left.name(&table) {
                if let Some(right_name) = env.right.name(&table) {
                    if left_name.range().contains(pos) || right_name.range().contains(pos) {
                        return Some((left_name, right_name));
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;
    use indoc::indoc;

    #[tokio::test]
    async fn environment() {
        let actual_edit = FeatureTester::new()
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \begin{foo}
                        \end{bar}
                    "#
                ),
            )
            .main("main.tex")
            .position(0, 8)
            .new_name("baz")
            .test_rename(LatexEnvironmentRenameProvider)
            .await
            .unwrap();

        let mut expected_changes = HashMap::new();
        expected_changes.insert(
            FeatureTester::uri("main.tex").into(),
            vec![
                TextEdit::new(Range::new_simple(0, 7, 0, 10), "baz".into()),
                TextEdit::new(Range::new_simple(1, 5, 1, 8), "baz".into()),
            ],
        );

        assert_eq!(actual_edit, WorkspaceEdit::new(expected_changes));
    }

    #[tokio::test]
    async fn command() {
        let actual_edit = FeatureTester::new()
            .file(
                "main.tex",
                indoc!(
                    r#"
                    \begin{foo}
                    \end{bar}
                "#
                ),
            )
            .main("main.tex")
            .position(0, 5)
            .new_name("baz")
            .test_rename(LatexEnvironmentRenameProvider)
            .await;

        assert_eq!(actual_edit, None);
    }

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_edit = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .new_name("")
            .test_rename(LatexEnvironmentRenameProvider)
            .await;

        assert_eq!(actual_edit, None);
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_edit = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .new_name("")
            .test_rename(LatexEnvironmentRenameProvider)
            .await;

        assert_eq!(actual_edit, None);
    }
}
