use crate::completion::factory;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::FeatureRequest;
use crate::syntax::latex::analysis::label::*;
use crate::syntax::latex::ast::LatexVisitor;
use crate::workspace::SyntaxTree;
use lsp_types::{CompletionItem, CompletionParams};

pub struct LatexLabelCompletionProvider;

impl LatexLabelCompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        await!(LatexCombinators::argument(
            request,
            &LABEL_REFERENCE_COMMANDS,
            0,
            async move |_| {
                let mut items = Vec::new();
                for document in &request.related_documents {
                    if let SyntaxTree::Latex(tree) = &document.tree {
                        let mut analyzer = LatexLabelAnalyzer::new();
                        analyzer.visit_root(&tree.root);
                        analyzer
                            .labels
                            .iter()
                            .filter(|label| label.kind == LatexLabelKind::Definition)
                            .map(|label| label.name.text().to_owned())
                            .map(factory::create_label)
                            .for_each(|item| items.push(item))
                    }
                }
                items
            }
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;
    use crate::workspace::WorkspaceBuilder;
    use futures::executor::block_on;

    #[test]
    fn test_inside_of_ref() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document(
            "foo.tex",
            "\\addbibresource{bar.bib}\\include{baz}\n\\ref{}",
        );
        builder.document("bar.bib", "");
        builder.document("baz.tex", "\\label{foo}\\label{bar}\\ref{baz}");
        let request = FeatureTester::new(builder.workspace, uri, 1, 5, "").into();

        let items = block_on(LatexLabelCompletionProvider::execute(&request));

        let labels: Vec<&str> = items.iter().map(|item| item.label.as_ref()).collect();
        assert_eq!(vec!["foo", "bar"], labels);
    }

    #[test]
    fn test_outside_of_ref() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\include{bar}\\ref{}");
        builder.document("bar.tex", "\\label{foo}\\label{bar}");
        let request = FeatureTester::new(builder.workspace, uri, 1, 6, "").into();

        let items = block_on(LatexLabelCompletionProvider::execute(&request));

        assert_eq!(items, Vec::new());
    }
}
