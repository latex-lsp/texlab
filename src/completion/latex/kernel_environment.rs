use crate::completion::factory;
use crate::completion::factory::LatexComponentId;
use crate::completion::latex::combinators::LatexCombinators;
use crate::completion::latex::kernel_primitives::KERNEL_ENVIRONMENTS;
use crate::feature::FeatureRequest;
use lsp_types::{CompletionItem, CompletionParams};

pub struct LatexKernelEnvironmentCompletionProvider;

impl LatexKernelEnvironmentCompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        await!(LatexCombinators::environment(&request, async move |_| {
            KERNEL_ENVIRONMENTS
                .iter()
                .map(|name| {
                    factory::create_environment((*name).to_owned(), LatexComponentId::Kernel)
                })
                .collect()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;
    use crate::workspace::WorkspaceBuilder;
    use futures::executor::block_on;

    #[test]
    fn test_inside_of_empty_begin() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\begin{}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 7, "").into();

        let items = block_on(LatexKernelEnvironmentCompletionProvider::execute(&request));

        assert_eq!(true, items.iter().any(|item| item.label == "document"));
    }

    #[test]
    fn test_inside_of_nonempty_end() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\end{foo}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 6, "").into();

        let items = block_on(LatexKernelEnvironmentCompletionProvider::execute(&request));

        assert_eq!(true, items.iter().any(|item| item.label == "document"));
    }

    #[test]
    fn test_outside_of_empty_begin() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\begin{}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 6, "").into();

        let items = block_on(LatexKernelEnvironmentCompletionProvider::execute(&request));

        assert_eq!(items, Vec::new());
    }

    #[test]
    fn test_outside_of_empty_end() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\end{}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 6, "").into();

        let items = block_on(LatexKernelEnvironmentCompletionProvider::execute(&request));

        assert_eq!(items, Vec::new());
    }

    #[test]
    fn test_inside_of_other_command() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\foo{bar}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 6, "").into();

        let items = block_on(LatexKernelEnvironmentCompletionProvider::execute(&request));

        assert_eq!(items, Vec::new());
    }

    #[test]
    fn test_inside_second_argument() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\begin{foo}{bar}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 14, "").into();

        let items = block_on(LatexKernelEnvironmentCompletionProvider::execute(&request));

        assert_eq!(items, Vec::new());
    }
}
