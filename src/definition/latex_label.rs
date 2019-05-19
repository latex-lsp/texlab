use crate::feature::FeatureRequest;
use crate::syntax::latex::*;
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use crate::workspace::Document;
use lsp_types::{Location, TextDocumentPositionParams};

pub struct LatexLabelDefinitionProvider;

impl LatexLabelDefinitionProvider {
    pub async fn execute(request: &FeatureRequest<TextDocumentPositionParams>) -> Vec<Location> {
        if let Some(reference) = Self::find_reference(&request) {
            for document in &request.related_documents {
                if let Some(definition) = Self::find_definition(&document, &reference) {
                    return vec![definition];
                }
            }
        }
        Vec::new()
    }

    fn find_definition(document: &Document, reference: &str) -> Option<Location> {
        if let SyntaxTree::Latex(tree) = &document.tree {
            tree.labels
                .iter()
                .filter(|label| label.kind() == LatexLabelKind::Definition)
                .find(|label| label.name().text() == reference)
                .map(|label| Location::new(document.uri.clone(), label.name().range()))
        } else {
            None
        }
    }

    fn find_reference(request: &FeatureRequest<TextDocumentPositionParams>) -> Option<&str> {
        if let SyntaxTree::Latex(tree) = &request.document.tree {
            tree.labels
                .iter()
                .find(|label| label.name().range().contains(request.params.position))
                .map(|label| label.name().text())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::completion::LatexComponentDatabase;
    use crate::feature::FeatureSpec;
    use crate::test_feature;
    use lsp_types::{Position, Range};

    #[test]
    fn test_has_definition() {
        let locations = test_feature!(
            LatexLabelDefinitionProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\label{foo}"),
                    FeatureSpec::file("bar.tex", "\\label{foo}\n\\input{baz.tex}"),
                    FeatureSpec::file("baz.tex", "\\ref{foo}"),
                ],
                main_file: "baz.tex",
                position: Position::new(0, 5),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(
            locations,
            vec![Location::new(
                FeatureSpec::uri("bar.tex"),
                Range::new_simple(0, 7, 0, 10)
            )]
        );
    }

    #[test]
    fn test_no_definition_latex() {
        let locations = test_feature!(
            LatexLabelDefinitionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", ""),],
                main_file: "foo.tex",
                position: Position::new(0, 0),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(locations, Vec::new());
    }

    #[test]
    fn test_no_definition_bibtex() {
        let locations = test_feature!(
            LatexLabelDefinitionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", ""),],
                main_file: "foo.bib",
                position: Position::new(0, 0),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(locations, Vec::new());
    }
}
