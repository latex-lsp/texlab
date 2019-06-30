use crate::completion::factory;
use crate::completion::factory::LatexComponentId;
use crate::completion::latex::combinators;
use crate::data::completion::LatexComponent;
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams, Range, TextEdit};
use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexComponentCommandProvider;

impl FeatureProvider for LatexComponentCommandProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::command(request, async move |command| {
            component(
                request,
                command.short_name_range(),
                |comp| &comp.commands,
                factory::command,
            )
            .await
        })
        .await
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexComponentEnvironmentProvider;

impl FeatureProvider for LatexComponentEnvironmentProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::environment(request, async move |_, name_range| {
            component(
                request,
                name_range,
                |comp| &comp.environments,
                factory::environment,
            )
            .await
        })
        .await
    }
}

async fn component<S, F>(
    request: &FeatureRequest<CompletionParams>,
    edit_range: Range,
    selector: S,
    factory: F,
) -> Vec<CompletionItem>
where
    S: Fn(&LatexComponent) -> &Vec<String>,
    F: Fn(
        &FeatureRequest<CompletionParams>,
        Cow<'static, str>,
        TextEdit,
        &LatexComponentId,
    ) -> CompletionItem,
{
    let components = request
        .component_database
        .related_components(request.related_documents());

    let mut items = Vec::new();
    for component in components {
        let file_names = component.file_names.iter().map(AsRef::as_ref).collect();
        let id = LatexComponentId::Component(file_names);
        for primitive in selector(&component) {
            let text_edit = TextEdit::new(edit_range, primitive.clone().into());
            let item = factory(request, primitive.clone().into(), text_edit, &id);
            items.push(item);
        }
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::completion::LatexComponentDatabase;
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::Position;
    use std::sync::Arc;

    fn create_database() -> LatexComponentDatabase {
        let components = vec![Arc::new(LatexComponent {
            file_names: vec!["foo.sty".into()],
            references: Vec::new(),
            commands: vec!["bar".into()],
            environments: vec!["baz".into()],
        })];

        LatexComponentDatabase { components }
    }

    #[test]
    fn test_command() {
        let items = test_feature(
            LatexComponentCommandProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\usepackage{foo} \\ba")],
                main_file: "foo.tex",
                position: Position::new(0, 20),
                component_database: create_database(),
                ..FeatureSpec::default()
            },
        );

        assert_eq!(items.len(), 1);
        assert_eq!(&items[0].label, "bar");
        assert_eq!(&items[0].detail.clone().unwrap(), "foo.sty");
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(0, 18, 0, 20))
        );
    }

    #[test]
    fn test_environment() {
        let items = test_feature(
            LatexComponentEnvironmentProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.tex",
                    "\\usepackage{foo} \\begin{ba}",
                )],
                main_file: "foo.tex",
                position: Position::new(0, 26),
                component_database: create_database(),
                ..FeatureSpec::default()
            },
        );

        assert_eq!(items.len(), 1);
        assert_eq!(&items[0].label, "baz");
        assert_eq!(&items[0].detail.clone().unwrap(), "foo.sty");
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(0, 24, 0, 26))
        );
    }
}
