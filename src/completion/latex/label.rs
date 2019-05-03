use crate::completion::factory;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::FeatureRequest;
use crate::syntax::latex::*;
use crate::workspace::SyntaxTree;
use lsp_types::{CompletionItem, CompletionParams};

pub struct LatexLabelCompletionProvider;

impl LatexLabelCompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        await!(LatexCombinators::argument(
            request,
            &LABEL_REFERENCE_COMMANDS,
            0,
            async move |_| {
                let mut items = Vec::new();
                for document in &request.related_documents {
                    if let SyntaxTree::Latex(tree) = &document.tree {
                        let mut analyzer = LatexLabelAnalyzer::new();
                        analyzer.visit_root(&tree.root);
                        analyzer
                            .labels
                            .iter()
                            .filter(|label| label.kind == LatexLabelKind::Definition)
                            .map(|label| label.name.text().to_owned())
                            .map(factory::create_label)
                            .for_each(|item| items.push(item))
                    }
                }
                items
            }
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::completion::latex::data::types::LatexComponentDatabase;
    use crate::feature::FeatureSpec;
    use crate::test_feature;
    use lsp_types::Position;

    #[test]
    fn test_inside_of_ref() {
        let items = test_feature!(
            LatexLabelCompletionProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file(
                        "foo.tex",
                        "\\addbibresource{bar.bib}\\include{baz}\n\\ref{}"
                    ),
                    FeatureSpec::file("bar.bib", ""),
                    FeatureSpec::file("baz.tex", "\\label{foo}\\label{bar}\\ref{baz}")
                ],
                main_file: "foo.tex",
                position: Position::new(1, 5),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        let labels: Vec<&str> = items.iter().map(|item| item.label.as_ref()).collect();
        assert_eq!(labels, vec!["foo", "bar"]);
    }

    #[test]
    fn test_outside_of_ref() {
        let items = test_feature!(
            LatexLabelCompletionProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\include{bar}\\ref{}"),
                    FeatureSpec::file("bar.tex", "\\label{foo}\\label{bar}"),
                ],
                main_file: "foo.tex",
                position: Position::new(1, 6),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(items, Vec::new());
    }
}
