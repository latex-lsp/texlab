use super::combinators::{self, Parameter};
use crate::completion::{factory, DATABASE};
use crate::workspace::*;
use futures_boxed::boxed;
use texlab_protocol::{CompletionItem, CompletionParams, TextEdit};
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
    F: Fn(&FeatureRequest<CompletionParams>, String, TextEdit) -> CompletionItem,
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

    combinators::argument(request, parameters, |context| {
        async move {
            let resolver = request.distribution.resolver().await;
            DATABASE
                .components
                .iter()
                .flat_map(|comp| comp.file_names.iter())
                .chain(resolver.files_by_name.keys())
                .filter(|file_name| file_name.ends_with(extension))
                .map(|file_name| {
                    let stem = &file_name[0..file_name.len() - 4];
                    let text_edit = TextEdit::new(context.range, stem.to_owned());
                    factory(request, stem.into(), text_edit)
                })
                .collect()
        }
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use texlab_protocol::Position;

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
