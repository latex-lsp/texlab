use crate::completion::factory;
use crate::completion::latex::combinators::LatexCombinators;
use crate::completion::latex::kernel_primitives::KERNEL_COMMANDS;
use crate::feature::FeatureRequest;
use lsp_types::{CompletionItem, CompletionParams};

pub struct LatexKernelCommandCompletionProvider;

impl LatexKernelCommandCompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        await!(LatexCombinators::command(&request, async move |_| {
            KERNEL_COMMANDS
                .iter()
                .map(|name| factory::create_command((*name).to_owned(), None))
                .collect()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;
    use crate::workspace::WorkspaceBuilder;
    use futures::executor;

    #[test]
    fn test_end_of_command() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\use");
        let request = FeatureTester::new(builder.workspace, uri, 0, 4, "").into();

        let items = executor::block_on(LatexKernelCommandCompletionProvider::execute(&request));

        assert_eq!(true, items.iter().any(|item| item.label == "usepackage"))
    }

    #[test]
    fn test_start_of_command() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\use");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();

        let items = executor::block_on(LatexKernelCommandCompletionProvider::execute(&request));

        assert_eq!(items, Vec::new());
    }

    #[test]
    fn test_outside_of_command() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "{%\\use\n}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 4, "").into();

        let items = executor::block_on(LatexKernelCommandCompletionProvider::execute(&request));

        assert_eq!(items, Vec::new());
    }
}
