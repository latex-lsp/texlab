use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::latex::{LatexLabel, LatexSyntaxTree};
use crate::syntax::text::{Span, SyntaxNode};
use crate::syntax::SyntaxTree;
use futures_boxed::boxed;
use lsp_types::*;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexLabelPrepareRenameProvider;

impl FeatureProvider for LatexLabelPrepareRenameProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Range>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<TextDocumentPositionParams>,
    ) -> Option<Range> {
        find_label(&request.document().tree, request.params.position).map(Span::range)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexLabelRenameProvider;

impl FeatureProvider for LatexLabelRenameProvider {
    type Params = RenameParams;
    type Output = Option<WorkspaceEdit>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<RenameParams>,
    ) -> Option<WorkspaceEdit> {
        let name = find_label(&request.document().tree, request.params.position)?;
        let mut changes = HashMap::new();
        for document in request.related_documents() {
            if let SyntaxTree::Latex(tree) = &document.tree {
                let edits = tree
                    .labels
                    .iter()
                    .flat_map(LatexLabel::names)
                    .filter(|label| label.text() == name.text)
                    .map(|label| {
                        TextEdit::new(label.range(), Cow::from(request.params.new_name.clone()))
                    })
                    .collect();
                changes.insert(document.uri.clone(), edits);
            }
        }
        Some(WorkspaceEdit::new(changes))
    }
}

fn find_label(tree: &SyntaxTree, position: Position) -> Option<&Span> {
    if let SyntaxTree::Latex(tree) = tree {
        tree.labels
            .iter()
            .flat_map(LatexLabel::names)
            .find(|label| label.range().contains(position))
            .map(|label| &label.span)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::{Position, Range};

    #[test]
    fn test_label() {
        let edit = test_feature(
            LatexLabelRenameProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\label{foo}\n\\include{bar}"),
                    FeatureSpec::file("bar.tex", "\\ref{foo}"),
                    FeatureSpec::file("baz.tex", "\\ref{foo}"),
                ],
                main_file: "foo.tex",
                position: Position::new(0, 7),
                new_name: "bar",
                ..FeatureSpec::default()
            },
        );
        let mut changes = HashMap::new();
        changes.insert(
            FeatureSpec::uri("foo.tex"),
            vec![TextEdit::new(
                Range::new_simple(0, 7, 0, 10),
                Cow::from("bar"),
            )],
        );
        changes.insert(
            FeatureSpec::uri("bar.tex"),
            vec![TextEdit::new(
                Range::new_simple(0, 5, 0, 8),
                Cow::from("bar"),
            )],
        );
        assert_eq!(edit, Some(WorkspaceEdit::new(changes)));
    }

    #[test]
    fn test_command_args() {
        let edit = test_feature(
            LatexLabelRenameProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\foo{bar}")],
                main_file: "foo.tex",
                position: Position::new(0, 5),
                new_name: "baz",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(edit, None);
    }

    #[test]
    fn test_bibtex() {
        let edit = test_feature(
            LatexLabelRenameProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "")],
                main_file: "foo.bib",
                position: Position::new(0, 0),
                new_name: "baz",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(edit, None);
    }
}
