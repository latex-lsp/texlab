use crate::completion::factory;
use crate::completion::latex::combinators::{self, ArgumentLocation};
use crate::data::language::{LatexIncludeKind, LANGUAGE_OPTIONS};
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;
use std::ffi::OsStr;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexClassImportProvider;

impl FeatureProvider for LatexClassImportProvider {
    type Params = CompletionParams;
    type Output = Vec<Arc<CompletionItem>>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        import(request, LatexIncludeKind::Class, factory::create_class).await
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexPackageImportProvider;

impl FeatureProvider for LatexPackageImportProvider {
    type Params = CompletionParams;
    type Output = Vec<Arc<CompletionItem>>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        import(request, LatexIncludeKind::Package, factory::create_class).await
    }
}

async fn import<F>(
    request: &FeatureRequest<CompletionParams>,
    kind: LatexIncludeKind,
    factory: F,
) -> Vec<Arc<CompletionItem>>
where
    F: Fn(Cow<'static, str>) -> CompletionItem,
{
    let extension = if kind == LatexIncludeKind::Package {
        "sty"
    } else {
        "cls"
    };

    let locations = LANGUAGE_OPTIONS
        .include_commands
        .iter()
        .filter(|cmd| cmd.kind == kind)
        .map(|cmd| ArgumentLocation::new(&cmd.name, cmd.index));

    combinators::argument(request, locations, async move |_| {
        request
            .resolver
            .files_by_name
            .values()
            .filter(|file| file.extension().and_then(OsStr::to_str) == Some(extension))
            .flat_map(|file| file.file_stem().unwrap().to_str())
            .map(|name| factory(Cow::from(name.to_owned())))
            .map(Arc::new)
            .collect()
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use crate::tex::resolver::TexResolver;
    use lsp_types::Position;
    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::path::PathBuf;

    fn create_resolver() -> TexResolver {
        let mut files_by_name = HashMap::new();
        files_by_name.insert(OsString::from("foo.sty"), PathBuf::from("./foo.sty"));
        files_by_name.insert(OsString::from("bar.cls"), PathBuf::from("./bar.cls"));
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
    }
}
