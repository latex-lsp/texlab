use crate::completion::factory;
use crate::completion::factory::LatexComponentId;
use crate::completion::latex::kernel_primitives::KERNEL_COMMANDS;
use crate::feature::FeatureRequest;
use crate::syntax::bibtex::BibtexNode;
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;

pub struct BibtexKernelCommandCompletionProvider;

impl BibtexKernelCommandCompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        if let SyntaxTree::Bibtex(tree) = &request.document.tree {
            if let Some(BibtexNode::Command(command)) = tree.find(request.params.position).last() {
                if command.token.range().contains(request.params.position)
                    && command.token.start().character != request.params.position.character
                {
                    return Self::generate_items();
                }
            }
        }
        Vec::new()
    }

    fn generate_items() -> Vec<CompletionItem> {
        KERNEL_COMMANDS
            .iter()
            .map(|command| factory::create_command(Cow::from(*command), &LatexComponentId::Kernel))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::completion::LatexComponentDatabase;
    use crate::feature::FeatureSpec;
    use crate::test_feature;
    use lsp_types::Position;

    #[test]
    fn test_inside_command() {
        let items = test_feature!(
            BibtexKernelCommandCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar=\n\\}")],
                main_file: "foo.bib",
                position: Position::new(1, 1),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(items.len() > 0, true);
    }

    #[test]
    fn test_start_of_command() {
        let items = test_feature!(
            BibtexKernelCommandCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar=\n\\}")],
                main_file: "foo.bib",
                position: Position::new(1, 0),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(items.len() == 0, true);
    }

    #[test]
    fn test_inside_text() {
        let items = test_feature!(
            BibtexKernelCommandCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar=\n}")],
                main_file: "foo.bib",
                position: Position::new(1, 0),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(items.len() == 0, true);
    }

    #[test]
    fn test_latex() {
        let items = test_feature!(
            BibtexKernelCommandCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\")],
                main_file: "foo.tex",
                position: Position::new(0, 1),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(items.len() == 0, true);
    }
}
