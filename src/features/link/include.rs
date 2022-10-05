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

    let working_dir = request.workspace.working_dir(
        request
            .workspace
            .parents(&document)
            .next()
            .as_ref()
            .unwrap_or(&document),
    );

    for include in &data.extras.explicit_links {
        if let Some(target) = include
            .targets(&working_dir, &request.workspace.environment.resolver)
            .find_map(|uri| request.workspace.get(&uri))
        {
            results.push(LinkResult {
                range: include.stem_range,
                target: Arc::clone(target.uri()),
            });
        }
    }

    Some(())
}
