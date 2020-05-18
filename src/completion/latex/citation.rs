use super::combinators::{self, ArgumentContext, Parameter};
use crate::{
    completion::types::{Item, ItemData},
    feature::FeatureRequest,
    protocol::{BibtexFormattingOptions, CompletionParams},
    syntax::{bibtex, BibtexEntryTypeCategory, Structure, LANGUAGE_DATA},
    workspace::{Document, DocumentContent},
};
use once_cell::sync::Lazy;
use petgraph::graph::NodeIndex;
use regex::Regex;

pub async fn complete_latex_citations<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
) {
    let parameters = LANGUAGE_DATA.citation_commands.iter().map(|cmd| Parameter {
        name: &cmd.name[1..],
        index: cmd.index,
    });

    combinators::argument(req, parameters, |ctx| async move {
        for doc in req.related() {
            if let DocumentContent::Bibtex(tree) = &doc.content {
                for entry_node in tree.children(tree.root) {
                    if let Some(item) = make_item(ctx, doc, tree, entry_node) {
                        items.push(item);
                    }
                }
            }
        }
    })
    .await;
}

fn make_item<'a>(
    ctx: ArgumentContext,
    doc: &'a Document,
    tree: &'a bibtex::Tree,
    entry_node: NodeIndex,
) -> Option<Item<'a>> {
    let entry = tree.as_entry(entry_node)?;
    if entry.is_comment() {
        return None;
    }

    let key = entry.key.as_ref()?.text();
    let options = BibtexFormattingOptions::default();
    let params = bibtex::FormattingParams {
        insert_spaces: true,
        tab_size: 4,
        options: &options,
    };
    let entry_code = bibtex::format(tree, entry_node, params);
    let text = format!(
        "{} {}",
        &key,
        WHITESPACE_REGEX
            .replace_all(
                &entry_code
                    .replace('{', "")
                    .replace('}', "")
                    .replace(',', " ")
                    .replace('=', " "),
                " ",
            )
            .trim()
    );

    let ty = LANGUAGE_DATA
        .find_entry_type(&entry.ty.text()[1..])
        .map(|ty| Structure::Entry(ty.category))
        .unwrap_or_else(|| Structure::Entry(BibtexEntryTypeCategory::Misc));

    let item = Item::new(
        ctx.range,
        ItemData::Citation {
            uri: &doc.uri,
            key,
            text,
            ty,
        },
    );
    Some(item)
}

static WHITESPACE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("\\s+").unwrap());

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        feature::FeatureTester,
        protocol::{Range, RangeExt},
    };
    use indoc::indoc;

    #[tokio::test]
    async fn empty_latex_document() {
        let req = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_citations(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let req = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_citations(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn incomplete() {
        let req = FeatureTester::new()
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \addbibresource{main.bib}
                        \cite{
                        \begin{foo}
                        \end{bar}
                    "#
                ),
            )
            .file("main.bib", "@article{foo,}")
            .main("main.tex")
            .position(1, 6)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_citations(&req, &mut actual_items).await;

        assert_eq!(actual_items.len(), 1);
        assert_eq!(actual_items[0].data.label(), "foo");
        assert_eq!(actual_items[0].range, Range::new_simple(1, 6, 1, 6));
    }

    #[tokio::test]
    async fn empty_key() {
        let req = FeatureTester::new()
            .file(
                "foo.tex",
                indoc!(
                    r#"
                        \addbibresource{bar.bib}
                        \cite{}
                    "#
                ),
            )
            .file("bar.bib", "@article{foo,}")
            .file("baz.bib", "@article{bar,}")
            .main("foo.tex")
            .position(1, 6)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_citations(&req, &mut actual_items).await;

        assert_eq!(actual_items.len(), 1);
        assert_eq!(actual_items[0].data.label(), "foo");
        assert_eq!(actual_items[0].range, Range::new_simple(1, 6, 1, 6));
    }

    #[tokio::test]
    async fn single_key() {
        let req = FeatureTester::new()
            .file(
                "foo.tex",
                indoc!(
                    r#"
                    \addbibresource{bar.bib}
                    \cite{foo}
                "#
                ),
            )
            .file("bar.bib", "@article{foo,}")
            .file("baz.bib", "@article{bar,}")
            .main("foo.tex")
            .position(1, 6)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_citations(&req, &mut actual_items).await;

        assert_eq!(actual_items.len(), 1);
        assert_eq!(actual_items[0].data.label(), "foo");
        assert_eq!(actual_items[0].range, Range::new_simple(1, 6, 1, 9));
    }

    #[tokio::test]
    async fn second_key() {
        let req = FeatureTester::new()
            .file(
                "foo.tex",
                indoc!(
                    r#"
                    \addbibresource{bar.bib}
                    \cite{foo,}
                "#
                ),
            )
            .file("bar.bib", "@article{foo,}")
            .file("baz.bib", "@article{bar,}")
            .main("foo.tex")
            .position(1, 10)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_citations(&req, &mut actual_items).await;

        assert_eq!(actual_items.len(), 1);
        assert_eq!(actual_items[0].data.label(), "foo");
        assert_eq!(actual_items[0].range, Range::new_simple(1, 10, 1, 10));
    }

    #[tokio::test]
    async fn outside_cite() {
        let req = FeatureTester::new()
            .file(
                "foo.tex",
                indoc!(
                    r#"
                        \addbibresource{bar.bib}
                        \cite{}
                    "#
                ),
            )
            .file("bar.bib", "@article{foo,}")
            .file("baz.bib", "@article{bar,}")
            .main("foo.tex")
            .position(1, 7)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_citations(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }
}
