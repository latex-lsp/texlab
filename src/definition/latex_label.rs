use futures_boxed::boxed;
use std::sync::Arc;
use texlab_protocol::{LocationLink, RangeExt, TextDocumentPositionParams};
use texlab_symbol::build_section_tree;
use texlab_syntax::*;
use texlab_workspace::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexLabelDefinitionProvider;

impl FeatureProvider for LatexLabelDefinitionProvider {
    type Params = TextDocumentPositionParams;
    type Output = Vec<LocationLink>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut links = Vec::new();
        if let Some(reference) = Self::find_reference(&request) {
            for document in request.related_documents() {
                let workspace = Arc::clone(&request.view.workspace);
                let view = DocumentView::new(workspace, Arc::clone(&document));
                Self::find_definitions(&view, &reference, &mut links);
            }
        }
        links
    }
}

impl LatexLabelDefinitionProvider {
    fn find_definitions(
        view: &DocumentView,
        reference: &LatexToken,
        links: &mut Vec<LocationLink>,
    ) {
        if let SyntaxTree::Latex(tree) = &view.document.tree {
            let outline = Outline::from(view);
            let section_tree = build_section_tree(view, tree);
            for label in &tree.structure.labels {
                if label.kind == LatexLabelKind::Definition {
                    let context = OutlineContext::parse(view, label, &outline);
                    for name in label.names() {
                        if name.text() == reference.text() {
                            let target_range = if let Some(OutlineContextItem::Section { .. }) =
                                context.as_ref().map(|ctx| &ctx.item)
                            {
                                section_tree
                                    .find(reference.text())
                                    .map(|sec| sec.full_range)
                            } else {
                                context.as_ref().map(|ctx| ctx.range)
                            };

                            links.push(LocationLink {
                                origin_selection_range: Some(reference.range()),
                                target_uri: view.document.uri.clone().into(),
                                target_range: target_range.unwrap_or_else(|| label.range()),
                                target_selection_range: label.range(),
                            });
                        }
                    }
                }
            }
        }
    }

    fn find_reference(request: &FeatureRequest<TextDocumentPositionParams>) -> Option<&LatexToken> {
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            tree.structure
                .labels
                .iter()
                .flat_map(LatexLabel::names)
                .find(|label| label.range().contains(request.params.position))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use texlab_protocol::{Position, Range};

    #[test]
    fn has_definition() {
        let links = test_feature(
            LatexLabelDefinitionProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\label{foo}"),
                    FeatureSpec::file(
                        "bar.tex",
                        "\\begin{a}\\begin{b}\\label{foo}\\end{b}\\end{a}\n\\input{baz.tex}",
                    ),
                    FeatureSpec::file("baz.tex", "\\ref{foo}"),
                ],
                main_file: "baz.tex",
                position: Position::new(0, 5),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            links,
            vec![LocationLink {
                origin_selection_range: Some(Range::new_simple(0, 5, 0, 8)),
                target_uri: FeatureSpec::uri("bar.tex"),
                target_range: Range::new_simple(0, 18, 0, 29),
                target_selection_range: Range::new_simple(0, 18, 0, 29)
            }]
        );
    }

    #[test]
    fn no_definition_latex() {
        let links = test_feature(
            LatexLabelDefinitionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "")],
                main_file: "foo.tex",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            },
        );
        assert!(links.is_empty());
    }

    #[test]
    fn no_definition_bibtex() {
        let links = test_feature(
            LatexLabelDefinitionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "")],
                main_file: "foo.bib",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            },
        );
        assert!(links.is_empty());
    }
}
