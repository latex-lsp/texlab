use texlab_protocol::RangeExt;
use crate::syntax::*;
use crate::workspace::*;
use futures_boxed::boxed;
use texlab_protocol::*;
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
        let name = find_label(
            &request.document().tree,
            request.params.text_document_position.position,
        )?;
        let mut changes = HashMap::new();
        for document in request.related_documents() {
            if let SyntaxTree::Latex(tree) = &document.tree {
                let edits = tree
                    .structure
                    .labels
                    .iter()
                    .flat_map(LatexLabel::names)
                    .filter(|label| label.text() == name.text)
                    .map(|label| TextEdit::new(label.range(), request.params.new_name.clone()))
                    .collect();
                changes.insert(document.uri.clone().into(), edits);
            }
        }
        Some(WorkspaceEdit::new(changes))
    }
}

fn find_label(tree: &SyntaxTree, position: Position) -> Option<&Span> {
    if let SyntaxTree::Latex(tree) = tree {
        tree.structure
            .labels
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
    use texlab_protocol::{Position, Range};

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
            vec![TextEdit::new(Range::new_simple(0, 7, 0, 10), "bar".into())],
        );
        changes.insert(
            FeatureSpec::uri("bar.tex"),
            vec![TextEdit::new(Range::new_simple(0, 5, 0, 8), "bar".into())],
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
