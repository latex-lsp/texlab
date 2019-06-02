use crate::completion::factory;
use crate::completion::factory::LatexComponentId;
use crate::completion::latex::combinators::LatexCombinators;
use crate::completion::latex::kernel_primitives::KERNEL_COMMANDS;
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;

pub struct LatexKernelCommandCompletionProvider {
    items: Vec<CompletionItem>,
}

impl LatexKernelCommandCompletionProvider {
    pub fn new() -> Self {
        let items = KERNEL_COMMANDS
            .iter()
            .map(|name| Cow::from(*name))
            .map(|name| factory::create_command(name, &LatexComponentId::Kernel))
            .collect();
        Self { items }
    }
}

impl FeatureProvider for LatexKernelCommandCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<CompletionParams>,
    ) -> Vec<CompletionItem> {
        LatexCombinators::command(&request, async move |_| self.items.clone()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::Position;

    #[test]
    fn test_end_of_command() {
        let items = test_feature(
            LatexKernelCommandCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\use")],
                main_file: "foo.tex",
                position: Position::new(0, 4),
                ..FeatureSpec::default()
            },
        );
        assert!(items.iter().any(|item| item.label == "usepackage"));
    }

    #[test]
    fn test_start_of_command() {
        let items = test_feature(
            LatexKernelCommandCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\use")],
                main_file: "foo.tex",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }

    #[test]
    fn test_outside_of_command() {
        let items = test_feature(
            LatexKernelCommandCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "{%\\use}")],
                main_file: "foo.tex",
                position: Position::new(0, 4),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }
}
