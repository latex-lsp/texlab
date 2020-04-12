use super::combinators::{self, Parameter};
use crate::{
    completion::factory,
    feature::{FeatureProvider, FeatureRequest},
};
use futures_boxed::boxed;
use texlab_components::COMPONENT_DATABASE;
use texlab_protocol::{CompletionItem, CompletionParams, TextEdit};
use texlab_syntax::{LatexIncludeKind, LANGUAGE_DATA};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexClassImportProvider;

impl FeatureProvider for LatexClassImportProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        import(req, LatexIncludeKind::Class, factory::class).await
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexPackageImportProvider;

impl FeatureProvider for LatexPackageImportProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        import(req, LatexIncludeKind::Package, factory::package).await
    }
}

async fn import<F>(
    req: &FeatureRequest<CompletionParams>,
    kind: LatexIncludeKind,
    mut factory: F,
) -> Vec<CompletionItem>
where
    F: FnMut(&FeatureRequest<CompletionParams>, String, TextEdit) -> CompletionItem,
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
        .map(|cmd| Parameter {
            name: &cmd.name,
            index: cmd.index,
        });

    combinators::argument(req, parameters, |ctx| async move {
        let resolver = req.distro.0.resolver().await;
        COMPONENT_DATABASE
            .components
            .iter()
            .flat_map(|comp| comp.file_names.iter())
            .chain(resolver.files_by_name.keys())
            .filter(|file_name| file_name.ends_with(extension))
            .map(|file_name| {
                let stem = &file_name[0..file_name.len() - 4];
                let text_edit = TextEdit::new(ctx.range, stem.to_owned());
                factory(req, stem.into(), text_edit)
            })
            .collect()
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;

    #[tokio::test]
    async fn empty_latex_document_class() {
        let actual_items = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion(LatexClassImportProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document_class() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion(LatexClassImportProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_latex_document_package() {
        let actual_items = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion(LatexPackageImportProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document_package() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion(LatexPackageImportProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn class() {
        let actual_items = FeatureTester::new()
            .file("main.tex", r#"\documentclass{}"#)
            .main("main.tex")
            .position(0, 15)
            .test_completion(LatexClassImportProvider)
            .await;

        assert!(actual_items.iter().any(|item| item.label == "beamer"));
        assert!(actual_items.iter().all(|item| item.label != "amsmath"));
    }

    #[tokio::test]
    async fn package() {
        let actual_items = FeatureTester::new()
            .file("main.tex", r#"\usepackage{}"#)
            .main("main.tex")
            .position(0, 12)
            .test_completion(LatexPackageImportProvider)
            .await;

        assert!(actual_items.iter().all(|item| item.label != "beamer"));
        assert!(actual_items.iter().any(|item| item.label == "amsmath"));
    }
}
