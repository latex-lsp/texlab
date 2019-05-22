use crate::completion::factory;
use crate::completion::factory::LatexComponentId;
use crate::completion::latex::combinators::LatexCombinators;
use crate::completion::latex::kernel_primitives::KERNEL_COMMANDS;
use crate::feature::FeatureRequest;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;

pub struct LatexKernelCommandCompletionProvider;

impl LatexKernelCommandCompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        await!(LatexCombinators::command(&request, async move |_| {
            KERNEL_COMMANDS
                .iter()
                .map(|name| factory::create_command(Cow::from(*name), &LatexComponentId::Kernel))
                .collect()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureSpec;
    use crate::test_feature;
    use lsp_types::Position;

    #[test]
    fn test_end_of_command() {
        let items = test_feature!(
            LatexKernelCommandCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\use")],
                main_file: "foo.tex",
                position: Position::new(0, 4),
                ..FeatureSpec::default()
            }
        );
        assert_eq!(items.iter().any(|item| item.label == "usepackage"), true);
    }

    #[test]
    fn test_start_of_command() {
        let items = test_feature!(
            LatexKernelCommandCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\use")],
                main_file: "foo.tex",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            }
        );
        assert_eq!(items, Vec::new());
    }

    #[test]
    fn test_outside_of_command() {
        let items = test_feature!(
            LatexKernelCommandCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "{%\\use}")],
                main_file: "foo.tex",
                position: Position::new(0, 4),
                ..FeatureSpec::default()
            }
        );
        assert_eq!(items, Vec::new());
    }
}
