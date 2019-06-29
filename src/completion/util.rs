use crate::completion::factory::LatexComponentId;
use crate::feature::FeatureRequest;
use lsp_types::*;
use std::borrow::Cow;

pub fn make_kernel_items<F>(
    names: &[&'static str],
    request: &FeatureRequest<CompletionParams>,
    edit_range: Range,
    factory: F,
) -> Vec<CompletionItem>
where
    F: Fn(
        &FeatureRequest<CompletionParams>,
        Cow<'static, str>,
        TextEdit,
        &LatexComponentId,
    ) -> CompletionItem,
{
    let mut items = Vec::new();
    for name in names {
        let text_edit = TextEdit::new(edit_range, Cow::from(*name));
        let item = factory(
            request,
            Cow::from(*name),
            text_edit,
            &LatexComponentId::Kernel,
        );
        items.push(item);
    }
    items
}
