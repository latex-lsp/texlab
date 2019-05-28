use crate::completion::factory;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::FeatureRequest;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;
use std::ffi::OsStr;

pub struct LatexClassImportProvider;

impl LatexClassImportProvider {
    const COMMANDS: &'static [&'static str] = &["\\documentclass"];

    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        await!(import(
            request,
            Self::COMMANDS,
            "cls",
            factory::create_class
        ))
    }
}

pub struct LatexPackageImportProvider;

impl LatexPackageImportProvider {
    const COMMANDS: &'static [&'static str] = &["\\usepackage"];

    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        await!(import(
            request,
            Self::COMMANDS,
            "sty",
            factory::create_package
        ))
    }
}

pub async fn import<'a, F>(
    request: &'a FeatureRequest<CompletionParams>,
    commands: &'a [&str],
    extension: &'a str,
    factory: F,
) -> Vec<CompletionItem>
where
    F: Fn(Cow<'static, str>) -> CompletionItem,
{
    let items = request
        .resolver
        .files_by_name
        .values()
        .filter(|file| file.extension().and_then(OsStr::to_str) == Some(extension))
        .flat_map(|file| file.file_stem().unwrap().to_str())
        .map(|name| factory(Cow::from(name.to_owned())))
        .collect();

    await!(LatexCombinators::argument(
        request,
        &commands,
        0,
        async move |_| { items }
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureSpec;
    use crate::resolver::TexResolver;
    use crate::test_feature;
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
        let items = test_feature!(
            LatexClassImportProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\documentclass{}")],
                main_file: "foo.tex",
                position: Position::new(0, 15),
                resolver: create_resolver(),
                ..FeatureSpec::default()
            }
        );

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "bar");
    }

    #[test]
    fn test_package() {
        let items = test_feature!(
            LatexPackageImportProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\usepackage{}")],
                main_file: "foo.tex",
                position: Position::new(0, 12),
                resolver: create_resolver(),
                ..FeatureSpec::default()
            }
        );

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "foo");
    }
}
