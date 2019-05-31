use crate::completion::factory;
use crate::completion::factory::LatexComponentId;
use crate::completion::latex::kernel_primitives::KERNEL_COMMANDS;
use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::bibtex::BibtexNode;
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use futures::prelude::*;
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;

#[derive(Debug, PartialEq, Clone)]
pub struct BibtexKernelCommandCompletionProvider {
    items: Vec<CompletionItem>,
}

impl BibtexKernelCommandCompletionProvider {
    pub fn new() -> Self {
        let items = KERNEL_COMMANDS
            .iter()
            .map(|command| Cow::from(*command))
            .map(|command| factory::create_command(command, &LatexComponentId::Kernel))
            .collect();
        Self { items }
    }
}

impl FeatureProvider for BibtexKernelCommandCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<CompletionParams>,
    ) -> Vec<CompletionItem> {
        if let SyntaxTree::Bibtex(tree) = &request.document.tree {
            if let Some(BibtexNode::Command(command)) = tree.find(request.params.position).last() {
                if command.token.range().contains(request.params.position)
                    && command.token.start().character != request.params.position.character
                {
                    return self.items.clone();
                }
            }
        }
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::Position;

    #[test]
    fn test_inside_command() {
        let items = test_feature(
            BibtexKernelCommandCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar=\n\\}")],
                main_file: "foo.bib",
                position: Position::new(1, 1),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(items.len() > 0, true);
    }

    #[test]
    fn test_start_of_command() {
        let items = test_feature(
            BibtexKernelCommandCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar=\n\\}")],
                main_file: "foo.bib",
                position: Position::new(1, 0),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(items.len() == 0, true);
    }

    #[test]
    fn test_inside_text() {
        let items = test_feature(
            BibtexKernelCommandCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar=\n}")],
                main_file: "foo.bib",
                position: Position::new(1, 0),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(items.len() == 0, true);
    }

    #[test]
    fn test_latex() {
        let items = test_feature(
            BibtexKernelCommandCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\")],
                main_file: "foo.tex",
                position: Position::new(0, 1),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(items.len() == 0, true);
    }
}
