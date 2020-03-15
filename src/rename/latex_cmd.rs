use crate::{
    feature::{FeatureProvider, FeatureRequest},
    protocol::{
        Position, Range, RenameParams, TextDocumentPositionParams, TextEdit, WorkspaceEdit,
    },
    syntax::{latex, SyntaxNode},
    workspace::DocumentContent,
};
use futures_boxed::boxed;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexCommandPrepareRenameProvider;

impl FeatureProvider for LatexCommandPrepareRenameProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Range>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let pos = req.params.position;
        find_command(&req.current().content, pos).map(SyntaxNode::range)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexCommandRenameProvider;

impl FeatureProvider for LatexCommandRenameProvider {
    type Params = RenameParams;
    type Output = Option<WorkspaceEdit>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let pos = req.params.text_document_position.position;
        let cmd_name = find_command(&req.current().content, pos)?.name.text();
        let mut changes = HashMap::new();
        for doc in req.related() {
            if let DocumentContent::Latex(table) = &doc.content {
                let edits = table
                    .commands
                    .iter()
                    .filter_map(|node| table.tree.as_command(*node))
                    .filter(|cmd| cmd.name.text() == cmd_name)
                    .map(|cmd| {
                        TextEdit::new(cmd.name.range(), format!("\\{}", req.params.new_name))
                    })
                    .collect();
                changes.insert(doc.uri.clone().into(), edits);
            }
        }
        Some(WorkspaceEdit::new(changes))
    }
}

fn find_command(content: &DocumentContent, pos: Position) -> Option<&latex::Command> {
    if let DocumentContent::Latex(table) = &content {
        table.tree.find_command_by_short_name_range(pos)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{feature::FeatureTester, protocol::RangeExt};
    use indoc::indoc;

    #[tokio::test]
    async fn command() {
        let actual_edit = FeatureTester::new()
            .file(
                "foo.tex",
                indoc!(
                    r#"
                        \include{bar.tex}
                        \baz
                    "#
                ),
            )
            .file("bar.tex", r#"\baz"#)
            .main("foo.tex")
            .position(1, 2)
            .new_name("qux")
            .test_rename(LatexCommandRenameProvider)
            .await
            .unwrap();

        let mut expected_changes = HashMap::new();
        expected_changes.insert(
            FeatureTester::uri("foo.tex").into(),
            vec![TextEdit::new(Range::new_simple(1, 0, 1, 4), "\\qux".into())],
        );
        expected_changes.insert(
            FeatureTester::uri("bar.tex").into(),
            vec![TextEdit::new(Range::new_simple(0, 0, 0, 4), "\\qux".into())],
        );

        assert_eq!(actual_edit, WorkspaceEdit::new(expected_changes));
    }

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_edit = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .new_name("")
            .test_rename(LatexCommandRenameProvider)
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
            .test_rename(LatexCommandRenameProvider)
            .await;

        assert_eq!(actual_edit, None);
    }
}
