use async_trait::async_trait;
use texlab_feature::{DocumentContent, FeatureProvider, FeatureRequest};
use texlab_protocol::{CompletionItem, CompletionParams, RangeExt};
use texlab_syntax::{latex, SyntaxNode};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PreselectCompletionProvider<F>(pub F);

#[async_trait]
impl<F> FeatureProvider for PreselectCompletionProvider<F>
where
    F: FeatureProvider<Params = CompletionParams, Output = Vec<CompletionItem>> + Send + Sync,
{
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let pos = req.params.text_document_position.position;
        let mut items = self.0.execute(req).await;
        if let DocumentContent::Latex(table) = &req.current().content {
            for env in &table.environments {
                if let Some(name) = env.left.name(&table) {
                    let right_args = table
                        .extract_group(env.right.parent, latex::GroupKind::Group, 0)
                        .unwrap();
                    let right_args_range = table[right_args].range();
                    let cond1 = right_args_range.contains_exclusive(pos);
                    let cond2 = table
                        .as_group(right_args)
                        .and_then(|group| group.right.as_ref())
                        .is_none()
                        && right_args_range.contains(pos);

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
