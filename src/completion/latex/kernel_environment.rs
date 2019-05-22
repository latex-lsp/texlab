use crate::completion::factory;
use crate::completion::factory::LatexComponentId;
use crate::completion::latex::combinators::LatexCombinators;
use crate::completion::latex::kernel_primitives::KERNEL_ENVIRONMENTS;
use crate::feature::FeatureRequest;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;

pub struct LatexKernelEnvironmentCompletionProvider;

impl LatexKernelEnvironmentCompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        await!(LatexCombinators::environment(&request, async move |_| {
            KERNEL_ENVIRONMENTS
                .iter()
                .map(|name| {
                    factory::create_environment(Cow::from(*name), &LatexComponentId::Kernel)
                })
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
    fn test_inside_of_empty_begin() {
        let items = test_feature!(
            LatexKernelEnvironmentCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\begin{}")],
                main_file: "foo.tex",
                position: Position::new(0, 7),
                ..FeatureSpec::default()
            }
        );
        assert_eq!(items.iter().any(|item| item.label == "document"), true);
    }

    #[test]
    fn test_inside_of_nonempty_end() {
        let items = test_feature!(
            LatexKernelEnvironmentCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\end{foo}")],
                main_file: "foo.tex",
                position: Position::new(0, 6),
                ..FeatureSpec::default()
            }
        );
        assert_eq!(items.iter().any(|item| item.label == "document"), true);
    }

    #[test]
    fn test_outside_of_empty_begin() {
        let items = test_feature!(
            LatexKernelEnvironmentCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\begin{}")],
                main_file: "foo.tex",
                position: Position::new(0, 6),
                ..FeatureSpec::default()
            }
        );
        assert_eq!(items, Vec::new());
    }

    #[test]
    fn test_outside_of_empty_end() {
        let items = test_feature!(
            LatexKernelEnvironmentCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\end{}")],
                main_file: "foo.tex",
                position: Position::new(0, 6),
                ..FeatureSpec::default()
            }
        );
        assert_eq!(items, Vec::new());
    }

    #[test]
    fn test_inside_of_other_command() {
        let items = test_feature!(
            LatexKernelEnvironmentCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\foo{bar}")],
                main_file: "foo.tex",
                position: Position::new(0, 6),
                ..FeatureSpec::default()
            }
        );
        assert_eq!(items, Vec::new());
    }

    #[test]
    fn test_inside_second_argument() {
        let items = test_feature!(
            LatexKernelEnvironmentCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\begin{foo}{bar}")],
                main_file: "foo.tex",
                position: Position::new(0, 14),
                ..FeatureSpec::default()
            }
        );
        assert_eq!(items, Vec::new());
    }
}
