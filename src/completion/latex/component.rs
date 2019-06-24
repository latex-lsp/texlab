use crate::completion::factory;
use crate::completion::factory::LatexComponentId;
use crate::completion::latex::combinators;
use crate::data::completion::LatexComponent;
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use itertools::Itertools;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexComponentCommandProvider;

impl FeatureProvider for LatexComponentCommandProvider {
    type Params = CompletionParams;
    type Output = Vec<Arc<CompletionItem>>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::command(request, async move |_| {
            component(request, |comp| &comp.commands, factory::create_command).await
        })
        .await
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexComponentEnvironmentProvider;

impl FeatureProvider for LatexComponentEnvironmentProvider {
    type Params = CompletionParams;
    type Output = Vec<Arc<CompletionItem>>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::environment(request, async move |_| {
            component(
                request,
                |comp| &comp.environments,
                factory::create_environment,
            )
            .await
        })
        .await
    }
}

async fn component<S, F>(
    request: &FeatureRequest<CompletionParams>,
    selector: S,
    factory: F,
) -> Vec<Arc<CompletionItem>>
where
    S: Fn(&LatexComponent) -> &Vec<String>,
    F: Fn(Cow<'static, str>, &LatexComponentId) -> CompletionItem,
{
    let components = request
        .component_database
        .related_components(request.related_documents());

    let mut items = Vec::new();
    for component in components {
        let file_names = component
            .file_names
            .clone()
            .into_iter()
            .map_into()
            .collect();
        let id = LatexComponentId::User(file_names);

        for primitive in selector(&component) {
            let item = factory(primitive.clone().into(), &id);
            items.push(Arc::new(item));
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
    }
}
