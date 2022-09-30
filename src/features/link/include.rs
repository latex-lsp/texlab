use std::sync::Arc;

use lsp_types::DocumentLinkParams;

use crate::features::FeatureRequest;

use super::LinkResult;

pub(super) fn find_include_links(
    request: &FeatureRequest<DocumentLinkParams>,
    results: &mut Vec<LinkResult>,
) -> Option<()> {
    let document = request.main_document();
    let data = document.data().as_latex()?;

    for include in &data.extras.explicit_links {
        for target in &include.targets {
            if request.workspace.get(&target).is_some() {
                results.push(LinkResult {
                    range: include.stem_range,
                    target: Arc::clone(target),
                });
                break;
            }
        }
    }

    Some(())
}
