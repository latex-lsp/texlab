use crate::data::language::LatexLabelKind;
use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::latex::{LatexLabel, LatexToken};
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use futures_boxed::boxed;
use lsp_types::{Location, ReferenceParams};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexLabelReferenceProvider;

impl FeatureProvider for LatexLabelReferenceProvider {
    type Params = ReferenceParams;
    type Output = Vec<Location>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<ReferenceParams>) -> Vec<Location> {
        let mut references = Vec::new();
        if let Some(definition) = Self::find_definition(request) {
            for document in request.related_documents() {
                if let SyntaxTree::Latex(tree) = &document.tree {
                    tree.labels
                        .iter()
                        .filter(|label| label.kind == LatexLabelKind::Reference)
                        .flat_map(LatexLabel::names)
                        .filter(|label| label.text() == definition)
                        .map(|label| Location::new(document.uri.clone(), label.range()))
                        .for_each(|location| references.push(location))
                }
            }
        }
        references
    }
}

impl LatexLabelReferenceProvider {
    fn find_definition(request: &FeatureRequest<ReferenceParams>) -> Option<&str> {
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            tree.labels
                .iter()
                .filter(|label| label.kind == LatexLabelKind::Definition)
                .flat_map(LatexLabel::names)
                .find(|label| label.range().contains(request.params.position))
                .map(LatexToken::text)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::{Position, Range};

    #[test]
    fn test() {
        let references = test_feature(
            LatexLabelReferenceProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\label{foo}"),
                    FeatureSpec::file("bar.tex", "\\input{foo.tex}\n\\ref{foo}"),
                    FeatureSpec::file("baz.tex", "\\ref{foo}"),
                ],
                main_file: "foo.tex",
                position: Position::new(0, 8),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            references,
            vec![Location::new(
                FeatureSpec::uri("bar.tex"),
                Range::new_simple(1, 5, 1, 8)
            )]
        );
    }

    #[test]
    fn test_bibtex() {
        let references = test_feature(
            LatexLabelReferenceProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "")],
                main_file: "foo.bib",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            },
        );
        assert!(references.is_empty());
    }
}
