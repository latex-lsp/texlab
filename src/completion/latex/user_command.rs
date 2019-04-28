use crate::completion::factory;
use crate::completion::factory::LatexComponentId;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::FeatureRequest;
use crate::syntax::latex::analysis::command::LatexCommandAnalyzer;
use crate::syntax::latex::ast::LatexVisitor;
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
    use crate::feature::FeatureTester;
    use crate::workspace::WorkspaceBuilder;
    use futures::executor;

    #[test]
    fn test() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\include{bar.tex}\n\\foo");
        builder.document("bar.tex", "\\bar");
        builder.document("baz.tex", "\\baz");
        let request = FeatureTester::new(builder.workspace, uri, 1, 2, "").into();

        let items = executor::block_on(LatexUserCommandCompletionProvider::execute(&request));

        let labels: Vec<&str> = items.iter().map(|item| item.label.as_ref()).collect();
        assert_eq!(vec!["include", "bar"], labels);
    }
}
