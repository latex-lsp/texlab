use crate::completion::factory;
use crate::completion::latex::combinators::{self, Parameter};
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams, TextEdit};
use texlab_completion_data::DATABASE;
use texlab_syntax::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexClassImportProvider;

impl FeatureProvider for LatexClassImportProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        import(request, LatexIncludeKind::Class, factory::class).await
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexPackageImportProvider;

impl FeatureProvider for LatexPackageImportProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        import(request, LatexIncludeKind::Package, factory::package).await
    }
}

async fn import<F>(
    request: &FeatureRequest<CompletionParams>,
    kind: LatexIncludeKind,
    factory: F,
) -> Vec<CompletionItem>
where
    F: Fn(&FeatureRequest<CompletionParams>, &'static str, TextEdit) -> CompletionItem,
{
    let extension = if kind == LatexIncludeKind::Package {
        "sty"
    } else {
        "cls"
    };

    let parameters = LANGUAGE_DATA
        .include_commands
        .iter()
        .filter(|cmd| cmd.kind == kind)
        .map(|cmd| Parameter::new(&cmd.name, cmd.index));

    combinators::argument(request, parameters, async move |context| {
        let mut items = Vec::new();
        for component in &DATABASE.components {
            for file_name in &component.file_names {
                if file_name.ends_with(extension) {
                    let stem = &file_name[0..file_name.len() - 4];
                    let text_edit = TextEdit::new(context.range, stem.into());
                    let item = factory(request, stem, text_edit);
                    items.push(item);
                }
            }
        }
        items
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::Position;

    #[test]
    fn test_class() {
        let items = test_feature(
            LatexClassImportProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\documentclass{}")],
                main_file: "foo.tex",
                position: Position::new(0, 15),
                ..FeatureSpec::default()
            },
        );

        assert!(items.iter().any(|item| item.label == "beamer"));
        assert!(items.iter().all(|item| item.label != "amsmath"));
    }

    #[test]
    fn test_package() {
        let items = test_feature(
            LatexPackageImportProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\usepackage{}")],
                main_file: "foo.tex",
                position: Position::new(0, 12),
                ..FeatureSpec::default()
            },
        );

        assert!(items.iter().all(|item| item.label != "beamer"));
        assert!(items.iter().any(|item| item.label == "amsmath"));
    }
}
