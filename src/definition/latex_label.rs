use crate::range::RangeExt;
use crate::syntax::*;
use crate::workspace::*;
use futures_boxed::boxed;
use lsp_types::{LocationLink, TextDocumentPositionParams};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexLabelDefinitionProvider;

impl FeatureProvider for LatexLabelDefinitionProvider {
    type Params = TextDocumentPositionParams;
    type Output = Vec<LocationLink>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut links = Vec::new();
        if let Some(reference) = Self::find_reference(&request) {
            for document in request.related_documents() {
                Self::find_definitions(&document, &reference, &mut links);
            }
        }
        links
    }
}

impl LatexLabelDefinitionProvider {
    fn find_definitions(
        document: &Document,
        reference: &LatexToken,
        links: &mut Vec<LocationLink>,
    ) {
        if let SyntaxTree::Latex(tree) = &document.tree {
            tree.labels
                .iter()
                .filter(|label| label.kind == LatexLabelKind::Definition)
                .flat_map(LatexLabel::names)
                .filter(|label| label.text() == reference.text())
                .map(|label| LocationLink {
                    origin_selection_range: Some(reference.range()),
                    target_uri: document.uri.clone().into(),
                    target_range: tree
                        .environments
                        .iter()
                        .find(|env| env.range().contains(label.start()))
                        .map(|env| env.range())
                        .unwrap_or_else(|| label.range()),
                    target_selection_range: label.range(),
                })
                .for_each(|link| links.push(link));
        }
    }

    fn find_reference(request: &FeatureRequest<TextDocumentPositionParams>) -> Option<&LatexToken> {
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            tree.labels
                .iter()
                .flat_map(LatexLabel::names)
                .find(|label| label.range().contains(request.params.position))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{Position, Range};

    #[test]
    fn test_has_definition() {
        let links = test_feature(
            LatexLabelDefinitionProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\label{foo}"),
                    FeatureSpec::file(
                        "bar.tex",
                        "\\begin{a}\\begin{b}\\label{foo}\\end{b}\\end{a}\n\\input{baz.tex}",
                    ),
                    FeatureSpec::file("baz.tex", "\\ref{foo}"),
                ],
                main_file: "baz.tex",
                position: Position::new(0, 5),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            links,
            vec![LocationLink {
                origin_selection_range: Some(Range::new_simple(0, 5, 0, 8)),
                target_uri: FeatureSpec::uri("bar.tex"),
                target_range: Range::new_simple(0, 9, 0, 36),
                target_selection_range: Range::new_simple(0, 25, 0, 28)
            }]
        );
    }

    #[test]
    fn test_no_definition_latex() {
        let links = test_feature(
            LatexLabelDefinitionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "")],
                main_file: "foo.tex",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            },
        );
        assert!(links.is_empty());
    }

    #[test]
    fn test_no_definition_bibtex() {
        let links = test_feature(
            LatexLabelDefinitionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "")],
                main_file: "foo.bib",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            },
        );
        assert!(links.is_empty());
    }
}
