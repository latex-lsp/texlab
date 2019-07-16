use crate::completion::factory::{self, LatexComponentId};
use crate::completion::latex::combinators;
use crate::data::completion::DATABASE;
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::*;
use lsp_types::{CompletionItem, CompletionParams};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexComponentCommandCompletionProvider;

impl FeatureProvider for LatexComponentCommandCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::command(request, async move |command| {
            let range = command.short_name_range();
            let mut items = Vec::new();
            for component in DATABASE.related_components(request.related_documents()) {
                let file_names = component.file_names.iter().map(AsRef::as_ref).collect();
                let id = LatexComponentId::Component(file_names);
                for command in &component.commands {
                    let text_edit = TextEdit::new(range, (&command.name).into());
                    let item = factory::command(
                        request,
                        (&command.name).into(),
                        command.image.as_ref().map(AsRef::as_ref),
                        text_edit,
                        &id,
                    );
                    items.push(item);
                }
            }
            items
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
            LatexComponentCommandCompletionProvider,
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
            LatexComponentCommandCompletionProvider,
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
            LatexComponentCommandCompletionProvider,
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
    fn test_command_package() {
        let items = test_feature(
            LatexComponentCommandCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\usepackage{lipsum}\n\\lips")],
                main_file: "foo.tex",
                position: Position::new(1, 2),
                ..FeatureSpec::default()
            },
        );
        assert!(items.iter().any(|item| item.label == "lipsum"));
    }

    #[test]
    fn test_command_package_comma_separated() {
        let items = test_feature(
            LatexComponentCommandCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.tex",
                    "\\usepackage{geometry, lipsum}\n\\lips",
                )],
                main_file: "foo.tex",
                position: Position::new(1, 2),
                ..FeatureSpec::default()
            },
        );
        assert!(items.iter().any(|item| item.label == "lipsum"));
    }

    #[test]
    fn test_command_class() {
        let items = test_feature(
            LatexComponentCommandCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.tex",
                    "\\documentclass{book}\n\\chap",
                )],
                main_file: "foo.tex",
                position: Position::new(1, 2),
                ..FeatureSpec::default()
            },
        );
        assert!(items.iter().any(|item| item.label == "chapter"));
    }

}
