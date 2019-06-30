use crate::completion::factory;
use crate::completion::latex::combinators::{self, Parameter};
use crate::data::language::{language_data, LatexIncludeKind};
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams, TextEdit};
use std::ffi::OsStr;

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

    let parameters = language_data()
        .include_commands
        .iter()
        .filter(|cmd| cmd.kind == kind)
        .map(|cmd| Parameter::new(&cmd.name, cmd.index));

    combinators::argument(request, parameters, async move |_, name_range| {
        request
            .resolver
            .files_by_name
            .values()
            .filter(|file| file.extension().and_then(OsStr::to_str) == Some(extension))
            .flat_map(|file| file.file_stem().unwrap().to_str())
            .map(|name| {
                factory(
                    request,
                    name.to_owned(),
                    TextEdit::new(name_range, name.to_owned().into()),
                )
            })
            .collect()
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use crate::tex::resolver::TexResolver;
    use lsp_types::{Position, Range};
    use std::collections::HashMap;

    fn create_resolver() -> TexResolver {
        let mut files_by_name = HashMap::new();
        files_by_name.insert("foo.sty".into(), "./foo.sty".into());
        files_by_name.insert("bar.cls".into(), "./bar.cls".into());
        TexResolver { files_by_name }
    }

    #[test]
    fn test_class() {
        let items = test_feature(
            LatexClassImportProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\documentclass{}")],
                main_file: "foo.tex",
                position: Position::new(0, 15),
                resolver: create_resolver(),
                ..FeatureSpec::default()
            },
        );

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "bar");
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(0, 15, 0, 15))
        );
    }

    #[test]
    fn test_package() {
        let items = test_feature(
            LatexPackageImportProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\usepackage{}")],
                main_file: "foo.tex",
                position: Position::new(0, 12),
                resolver: create_resolver(),
                ..FeatureSpec::default()
            },
        );

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "foo");
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(0, 12, 0, 12))
        );
    }
}
