use crate::completion::factory;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;
use std::ffi::OsStr;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexClassImportProvider;

impl LatexClassImportProvider {
    const COMMANDS: &'static [&'static str] = &["\\documentclass"];
}

impl FeatureProvider for LatexClassImportProvider {
    type Params = CompletionParams;
    type Output = Vec<Arc<CompletionItem>>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        import(request, Self::COMMANDS, "cls", factory::create_class).await
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexPackageImportProvider;

impl LatexPackageImportProvider {
    const COMMANDS: &'static [&'static str] = &["\\usepackage"];
}

impl FeatureProvider for LatexPackageImportProvider {
    type Params = CompletionParams;
    type Output = Vec<Arc<CompletionItem>>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        import(request, Self::COMMANDS, "sty", factory::create_class).await
    }
}

pub async fn import<'a, F>(
    request: &'a FeatureRequest<CompletionParams>,
    commands: &'a [&str],
    extension: &'a str,
    factory: F,
) -> Vec<Arc<CompletionItem>>
where
    F: Fn(Cow<'static, str>) -> CompletionItem,
{
    LatexCombinators::argument(request, &commands, 0, async move |_| {
        request
            .distribution
            .packages
            .iter()
            .flat_map(|package| &package.run_files)
            .filter(|file| file.extension().and_then(OsStr::to_str) == Some(extension))
            .map(|file| file.file_stem().unwrap().to_str().unwrap())
            .map(|name| Arc::new(factory(Cow::from(name.to_owned()))))
            .collect()
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distribution::{PackageManifest, TexDistribution};
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::Position;
    use std::path::PathBuf;

    fn create_distribution() -> TexDistribution {
        let packages = vec![
            PackageManifest {
                run_files: vec![PathBuf::from("./foo.sty")],
                ..PackageManifest::default()
            },
            PackageManifest {
                run_files: vec![PathBuf::from("./bar.cls")],
                ..PackageManifest::default()
            },
        ];

        TexDistribution {
            packages,
            ..TexDistribution::default()
        }
    }

    #[test]
    fn test_class() {
        let items = test_feature(
            LatexClassImportProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\documentclass{}")],
                main_file: "foo.tex",
                position: Position::new(0, 15),
                distribution: create_distribution(),
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
                distribution: create_distribution(),
                ..FeatureSpec::default()
            },
        );

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "foo");
    }
}
