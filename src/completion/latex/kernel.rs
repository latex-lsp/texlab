use crate::completion::factory;
use crate::completion::latex::combinators;
use crate::completion::util::make_kernel_items;
use crate::data::kernel_primitives::{KERNEL_COMMANDS, KERNEL_ENVIRONMENTS};
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexKernelCommandCompletionProvider;

impl FeatureProvider for LatexKernelCommandCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::command(request, async move |command| {
            make_kernel_items(
                KERNEL_COMMANDS,
                request,
                command.short_name_range(),
                factory::command,
            )
        })
        .await
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexKernelEnvironmentCompletionProvider;

impl FeatureProvider for LatexKernelEnvironmentCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::environment(request, async move |context| {
            make_kernel_items(
                KERNEL_ENVIRONMENTS,
                request,
                context.range,
                factory::environment,
            )
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::{Position, Range};

    #[test]
    fn test_command_start() {
        let items = test_feature(
            LatexKernelCommandCompletionProvider,
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
    fn test_command_end() {
        let items = test_feature(
            LatexKernelCommandCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\use")],
                main_file: "foo.tex",
                position: Position::new(0, 4),
                ..FeatureSpec::default()
            },
        );
        assert!(!items.is_empty());
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(0, 1, 0, 4))
        );
    }

    #[test]
    fn test_command_word() {
        let items = test_feature(
            LatexKernelCommandCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "use")],
                main_file: "foo.tex",
                position: Position::new(0, 2),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }

    #[test]
    fn test_environment_inside_of_empty_begin() {
        let items = test_feature(
            LatexKernelEnvironmentCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\begin{}")],
                main_file: "foo.tex",
                position: Position::new(0, 7),
                ..FeatureSpec::default()
            },
        );
        assert!(!items.is_empty());
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(0, 7, 0, 7))
        );
    }

    #[test]
    fn test_environment_inside_of_non_empty_end() {
        let items = test_feature(
            LatexKernelEnvironmentCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\end{foo}")],
                main_file: "foo.tex",
                position: Position::new(0, 6),
                ..FeatureSpec::default()
            },
        );
        assert!(!items.is_empty());
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(0, 5, 0, 8))
        );
    }

    #[test]
    fn test_environment_outside_of_empty_begin() {
        let items = test_feature(
            LatexKernelEnvironmentCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\begin{}")],
                main_file: "foo.tex",
                position: Position::new(0, 6),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }

    #[test]
    fn test_environment_outside_of_empty_end() {
        let items = test_feature(
            LatexKernelEnvironmentCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\end{}")],
                main_file: "foo.tex",
                position: Position::new(0, 6),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }

    #[test]
    fn test_environment_inside_of_other_command() {
        let items = test_feature(
            LatexKernelEnvironmentCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\foo{bar}")],
                main_file: "foo.tex",
                position: Position::new(0, 6),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }

    #[test]
    fn test_environment_inside_second_argument() {
        let items = test_feature(
            LatexKernelEnvironmentCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\begin{foo}{bar}")],
                main_file: "foo.tex",
                position: Position::new(0, 14),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }

    #[test]
    fn test_environment_unterminated() {
        let items = test_feature(
            LatexKernelEnvironmentCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\begin{ foo")],
                main_file: "foo.tex",
                position: Position::new(0, 7),
                ..FeatureSpec::default()
            },
        );
        assert!(!items.is_empty());
    }
}
