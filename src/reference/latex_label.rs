use crate::feature::FeatureRequest;
use crate::range;
use crate::syntax::latex::analysis::label::{LatexLabelAnalyzer, LatexLabelKind};
use crate::syntax::latex::ast::LatexVisitor;
use crate::workspace::SyntaxTree;
use lsp_types::{Location, ReferenceParams};

pub struct LatexLabelReferenceProvider;

impl LatexLabelReferenceProvider {
    pub async fn execute(request: &FeatureRequest<ReferenceParams>) -> Vec<Location> {
        let mut references = Vec::new();
        if let Some(definition) = Self::find_definition(request) {
            for document in &request.related_documents {
                if let SyntaxTree::Latex(tree) = &document.tree {
                    let mut analyzer = LatexLabelAnalyzer::new();
                    analyzer.visit_root(&tree.root);
                    analyzer
                        .labels
                        .iter()
                        .filter(|label| label.kind == LatexLabelKind::Reference)
                        .filter(|label| label.name.text() == definition)
                        .map(|label| Location::new(document.uri.clone(), label.command.range))
                        .for_each(|location| references.push(location))
                }
            }
        }
        references
    }

    fn find_definition(request: &FeatureRequest<ReferenceParams>) -> Option<&str> {
        if let SyntaxTree::Latex(tree) = &request.document.tree {
            let mut analyzer = LatexLabelAnalyzer::new();
            analyzer.visit_root(&tree.root);
            analyzer
                .labels
                .iter()
                .find(|label| {
                    label.kind == LatexLabelKind::Definition
                        && range::contains(label.command.range, request.params.position)
                })
                .map(|label| label.name.text())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::completion::latex::data::types::LatexComponentDatabase;
    use crate::feature::FeatureSpec;
    use crate::range;
    use crate::test_feature;
    use lsp_types::Position;

    #[test]
    fn test() {
        let references = test_feature!(
            LatexLabelReferenceProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\label{foo}"),
                    FeatureSpec::file("bar.tex", "\\input{foo.tex}\n\\ref{foo}"),
                    FeatureSpec::file("baz.tex", "\\ref{foo}")
                ],
                main_file: "foo.tex",
                position: Position::new(0, 8),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(
            references,
            vec![Location::new(
                FeatureSpec::uri("bar.tex"),
                range::create(1, 0, 1, 9)
            )]
        );
    }

    #[test]
    fn test_bibtex() {
        let references = test_feature!(
            LatexLabelReferenceProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", ""),],
                main_file: "foo.bib",
                position: Position::new(0, 0),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(references, Vec::new());
    }
}
