use crate::feature::{FeatureProvider, FeatureRequest};
use texlab_syntax::*;
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};

#[derive(Debug)]
pub struct PreselectCompletionProvider<F> {
    provider: F,
}

impl<F> PreselectCompletionProvider<F> {
    pub fn new(provider: F) -> Self {
        Self { provider }
    }
}

impl<F> FeatureProvider for PreselectCompletionProvider<F>
where
    F: FeatureProvider<Params = CompletionParams, Output = Vec<CompletionItem>> + Send + Sync,
{
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut items = self.provider.execute(request).await;
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            for environment in &tree.environments {
                if let Some(name) = environment.left.name() {
                    let right_args = &environment.right.command.args[0];
                    let cond1 = right_args
                        .range()
                        .contains_exclusive(request.params.position);
                    let cond2 = right_args.right.is_none()
                        && right_args.range().contains(request.params.position);

                    if cond1 || cond2 {
                        for item in &mut items {
                            item.preselect = Some(false);
                            if item.label == name.text() {
                                item.preselect = Some(true);
                                break;
                            }
                        }
                    }
                }
            }
        }

        items
    }
}
