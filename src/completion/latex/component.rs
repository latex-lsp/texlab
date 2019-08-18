use super::combinators;
use crate::completion::factory::{self, LatexComponentId, LatexDocumentation};
use crate::completion::DATABASE;
use crate::workspace::*;
use futures_boxed::boxed;
use lsp_types::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexComponentCommandCompletionProvider;

impl FeatureProvider for LatexComponentCommandCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let supports_images: bool = request
            .client_capabilities
            .text_document
            .as_ref()
            .and_then(|cap| cap.completion.as_ref())
            .and_then(|cap| cap.completion_item.as_ref())
            .and_then(|cap| cap.documentation_format.as_ref())
            .map_or(true, |formats| formats.contains(&MarkupKind::Markdown));
        combinators::command(request, async move |command| {
            let range = command.short_name_range();
            let mut items = Vec::new();
            for component in DATABASE.related_components(request.related_documents()) {
                let file_names = component.file_names.iter().map(AsRef::as_ref).collect();
                let id = LatexComponentId::Component(file_names);
                for command in &component.commands {
                    let text_edit = TextEdit::new(range, (&command.name).into());
                    let doc: LatexDocumentation = if supports_images {
                        match &command.image {
                            None => LatexDocumentation::None,
                            Some(image) => LatexDocumentation::Image(&image),
                        }
                    } else {
                        match &command.glyph {
                            None => LatexDocumentation::None,
                            Some(glyph) => LatexDocumentation::Glyph(&glyph),
                        }
                    };
                    let item = factory::command(
                        request,
                        (&command.name).into(),
                        doc,
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexComponentEnvironmentCompletionProvider;

impl FeatureProvider for LatexComponentEnvironmentCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::environment(request, async move |context| {
            let mut items = Vec::new();
            for component in DATABASE.related_components(request.related_documents()) {
                let file_names = component.file_names.iter().map(AsRef::as_ref).collect();
                let id = LatexComponentId::Component(file_names);
                for environment in &component.environments {
                    let text_edit = TextEdit::new(context.range, environment.into());
                    let item = factory::environment(request, environment.into(), text_edit, &id);
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

    #[test]
    fn test_environment_inside_of_empty_begin() {
        let items = test_feature(
            LatexComponentEnvironmentCompletionProvider,
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
            LatexComponentEnvironmentCompletionProvider,
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
            LatexComponentEnvironmentCompletionProvider,
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
            LatexComponentEnvironmentCompletionProvider,
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
            LatexComponentEnvironmentCompletionProvider,
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
            LatexComponentEnvironmentCompletionProvider,
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
            LatexComponentEnvironmentCompletionProvider,
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
