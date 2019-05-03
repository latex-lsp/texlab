use crate::completion::factory;
use crate::completion::factory::LatexComponentId;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::FeatureRequest;
use crate::syntax::latex::{LatexCommandAnalyzer, LatexVisitor};
use crate::syntax::text::SyntaxNode;
use crate::workspace::SyntaxTree;
use itertools::Itertools;
use lsp_types::{CompletionItem, CompletionParams};

pub struct LatexUserCommandCompletionProvider;

impl LatexUserCommandCompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        await!(LatexCombinators::command(
            request,
            async move |current_command| {
                let mut items = Vec::new();
                for document in &request.related_documents {
                    if let SyntaxTree::Latex(tree) = &document.tree {
                        let mut analyzer = LatexCommandAnalyzer::new();
                        analyzer.visit_root(&tree.root);
                        analyzer
                            .commands
                            .iter()
                            .filter(|command| command.range() != current_command.range())
                            .map(|command| &command.name.text()[1..])
                            .unique()
                            .map(|name| {
                                factory::create_command(name.to_owned(), LatexComponentId::Unknown)
                            })
                            .for_each(|item| items.push(item));
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
    fn test() {
        let items = test_feature!(
            LatexUserCommandCompletionProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\include{bar.tex}\n\\foo"),
                    FeatureSpec::file("bar.tex", "\\bar"),
                    FeatureSpec::file("baz.tex", "\\baz")
                ],
                main_file: "foo.tex",
                position: Position::new(1, 2),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        let labels: Vec<&str> = items.iter().map(|item| item.label.as_ref()).collect();
        assert_eq!(labels, vec!["include", "bar"]);
    }
}
