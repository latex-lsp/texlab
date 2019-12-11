use crate::workspace::*;
use futures_boxed::boxed;
use texlab_protocol::RangeExt;
use texlab_protocol::{CompletionItem, CompletionParams};
use texlab_syntax::*;

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
            for environment in &tree.env.environments {
                if let Some(name) = environment.left.name() {
                    let right_args = &environment.right.command.args[0];
                    let cond1 = right_args
                        .range()
                        .contains_exclusive(request.params.text_document_position.position);
                    let cond2 = right_args.right.is_none()
                        && right_args
                            .range()
                            .contains(request.params.text_document_position.position);

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
