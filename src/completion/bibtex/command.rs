use crate::completion::factory;
use crate::completion::util::make_kernel_items;
use crate::data::kernel_primitives::KERNEL_COMMANDS;
use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::bibtex::BibtexNode;
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BibtexCommandCompletionProvider;

impl FeatureProvider for BibtexCommandCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        if let SyntaxTree::Bibtex(tree) = &request.document().tree {
            let position = request.params.position;
            if let Some(BibtexNode::Command(command)) = tree.find(position).last() {
                if command.token.range().contains(position)
                    && command.token.start().character != position.character
                {
                    let mut range = command.range();
                    range.start.character += 1;
                    return make_kernel_items(KERNEL_COMMANDS, request, range, factory::command);
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
    use lsp_types::{Position, Range};

    #[test]
    fn test_inside_command() {
        let items = test_feature(
            BibtexCommandCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar=\n\\}")],
                main_file: "foo.bib",
                position: Position::new(1, 1),
                ..FeatureSpec::default()
            },
        );
        assert!(!items.is_empty());
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(1, 1, 1, 2))
        );
    }

    #[test]
    fn test_start_of_command() {
        let items = test_feature(
            BibtexCommandCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar=\n\\}")],
                main_file: "foo.bib",
                position: Position::new(1, 0),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }

    #[test]
    fn test_inside_text() {
        let items = test_feature(
            BibtexCommandCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar=\n}")],
                main_file: "foo.bib",
                position: Position::new(1, 0),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }

    #[test]
    fn test_latex() {
        let items = test_feature(
            BibtexCommandCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\")],
                main_file: "foo.tex",
                position: Position::new(0, 1),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }
}
