use super::combinators::{self, Parameter};
use crate::{
    completion::types::{Item, ItemData},
    components::COMPONENT_DATABASE,
    feature::FeatureRequest,
    protocol::CompletionParams,
    syntax::{LatexIncludeKind, LANGUAGE_DATA},
};
use std::{borrow::Cow, collections::HashSet};

pub async fn complete_latex_classes<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
) {
    complete_latex_imports(req, items, LatexIncludeKind::Class, |name| {
        ItemData::Class { name }
    })
    .await;
}

pub async fn complete_latex_packages<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
) {
    complete_latex_imports(req, items, LatexIncludeKind::Package, |name| {
        ItemData::Package { name }
    })
    .await;
}

async fn complete_latex_imports<'a, F>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
    kind: LatexIncludeKind,
    mut factory: F,
) where
    F: FnMut(Cow<'a, str>) -> ItemData<'a>,
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
            name: &cmd.name[1..],
            index: cmd.index,
        });

    combinators::argument(req, parameters, |ctx| async move {
        let resolver = req.distro.resolver().await;
        let mut file_names = HashSet::new();
        COMPONENT_DATABASE
            .components
            .iter()
            .flat_map(|comp| comp.file_names.iter())
            .filter(|file_name| file_name.ends_with(extension))
            .for_each(|file_name| {
                file_names.insert(file_name);
                let stem = &file_name[0..file_name.len() - 4];
                let data = factory(stem.into());
                let item = Item::new(ctx.range, data);
                items.push(item);
            });

        resolver
            .files_by_name
            .keys()
            .filter(|file_name| file_name.ends_with(extension) && !file_names.contains(file_name))
            .for_each(|file_name| {
                let stem = &file_name[0..file_name.len() - 4];
                let data = factory(stem.to_owned().into());
                let item = Item::new(ctx.range, data);
                items.push(item);
            });
    })
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;

    #[tokio::test]
    async fn empty_latex_document_class() {
        let req = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_classes(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document_class() {
        let req = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_classes(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_latex_document_package() {
        let req = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_packages(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document_package() {
        let req = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_packages(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn class() {
        let req = FeatureTester::new()
            .file("main.tex", r#"\documentclass{}"#)
            .main("main.tex")
            .position(0, 15)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_classes(&req, &mut actual_items).await;

        assert!(actual_items
            .iter()
            .any(|item| item.data.label() == "beamer"));
        assert!(actual_items
            .iter()
            .all(|item| item.data.label() != "amsmath"));
    }

    #[tokio::test]
    async fn package() {
        let req = FeatureTester::new()
            .file("main.tex", r#"\usepackage{}"#)
            .main("main.tex")
            .position(0, 12)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_packages(&req, &mut actual_items).await;

        assert!(actual_items
            .iter()
            .all(|item| item.data.label() != "beamer"));
        assert!(actual_items
            .iter()
            .any(|item| item.data.label() == "amsmath"));
    }
}
